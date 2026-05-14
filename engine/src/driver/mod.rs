pub mod postgres;
pub mod sqlite;

use crate::database_config::{DatabaseConfig, DatabaseKind};
use crate::error::EngineError;
use crate::result::QueryResult;
use crate::schema::{DataChangeset, TableBrief, TableInfo};

/// Cung cấp các quy tắc định dạng SQL (Dialect) cho từng loại Database
///
/// Triển khai trait này cho từng loại Database để định dạng SQL phù hợp, cho phép sinh script chuẩn
/// cho các thao tác như tạo bảng, thêm dữ liệu, cập nhật, xóa.
pub trait SqlDialect {
    /// Bọc định dang (tên bảng, tên cột)
    ///
    /// # Example
    /// - `"name"`: (SQLite, PostgreSQL)
    /// - `[name]`: (MSSQL)
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Định dạng giá trị dựa trên kiểu dữ liệu
    fn format_value(&self, value: &str, data_type: &str) -> String;
}

/// Trait đại diện cho một database driver.
///
/// **Chỉ chứa usage methods** — KHÔNG chứa creation/connect logic.
/// Việc tạo driver được xử lý bởi [`create`] factory function,
/// giúp tách biệt concerns và cho phép dynamic dispatch (`dyn DatabaseDriver`).
///
/// `SqlClient` bọc trait này và cung cấp API đơn giản cho caller.
///
/// # Implement cho database mới
///
/// 1. Tạo file `driver/mydb.rs`
/// 2. Implement trait này cho struct của bạn
/// 3. Thêm case vào [`create`] factory function
/// 4. Thêm variant vào [`DatabaseKind`]
#[async_trait::async_trait]
pub trait DatabaseDriver: SqlDialect + Send + Sync {
    // =======================
    // =   Query Execution   =
    // =======================

    /// Thực thi SQL query bất kỳ (DDL, DML, DQL).
    async fn execute(&self, query: &str) -> Result<QueryResult, EngineError>;

    /// Kiểm tra kết nối còn sống không.
    async fn ping(&self) -> Result<(), EngineError>;

    /// Tên loại database (ví dụ: "SQLite", "PostgreSQL").
    fn database_type(&self) -> &'static str;

    /// Sinh SQL script từ data-changeset (DML).
    fn generate_changeset_script(&self, changeset: &DataChangeset) -> String {
        let mut script = String::new();

        for update in &changeset.updates {
            let set_clause = update
                .changes
                .iter()
                .map(|c| {
                    format!(
                        "{} = {}",
                        self.quote_identifier(&c.column_name),
                        self.format_value(&c.value, &c.data_type)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

            let where_clause = update
                .pk_conditions
                .iter()
                .map(|c| {
                    format!(
                        "{} = {}",
                        self.quote_identifier(&c.column_name),
                        self.format_value(&c.value, &c.data_type)
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ");

            script.push_str(&format!(
                "UPDATE {} SET {} WHERE {};\n",
                self.quote_identifier(&changeset.table_name),
                set_clause,
                where_clause
            ));
        }

        for delete in &changeset.deletes {
            let where_clause = delete
                .pk_conditions
                .iter()
                .map(|c| {
                    format!(
                        "{} = {}",
                        self.quote_identifier(&c.column_name),
                        self.format_value(&c.value, &c.data_type)
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ");

            script.push_str(&format!(
                "DELETE FROM {} WHERE {};\n",
                self.quote_identifier(&changeset.table_name),
                where_clause
            ));
        }

        script
    }

    // ============================
    // =   Schema Introspection   =
    // ============================

    /// Liệt kê tất cả tables và views trong database.
    async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError>;

    /// Liệt kê tất cả views trong database.
    async fn list_views(&self) -> Result<Vec<String>, EngineError>;

    /// Lấy thông tin chi tiết của một table.
    async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError>;

    /// Đếm số dòng trong table.
    async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError>;
}

/// Entry point duy nhất để tạo driver mới.
///
/// Tự động chọn driver phù hợp dựa trên `config.kind()`.
pub async fn create(config: &DatabaseConfig) -> Result<Box<dyn DatabaseDriver>, EngineError> {
    match config.kind() {
        DatabaseKind::Sqlite => {
            let driver = sqlite::SqliteDriver::new(config).await?;
            Ok(Box::new(driver))
        }
        DatabaseKind::Postgres => {
            let driver = postgres::PostgresDriver::new(config).await?;
            Ok(Box::new(driver))
        }
    }
}
