pub mod sqlite;

use crate::config::{DatabaseConfig, DatabaseKind};
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
    /// - `"name"`: (SQLite)
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
///
/// ```ignore
/// pub struct PostgresDriver {
///     pool: PgPool,
/// }
///
/// #[async_trait::async_trait]
/// impl DatabaseDriver for PostgresDriver {
///     async fn execute(&self, query: &str) -> Result<QueryResult, EngineError> { ... }
///     async fn ping(&self) -> Result<(), EngineError> { ... }
///     fn database_type(&self) -> &'static str { "PostgreSQL" }
///     async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError> { ... }
///     async fn list_views(&self) -> Result<Vec<String>, EngineError> { ... }
///     async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError> { ... }
///     async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError> { ... }
/// }
/// ```
#[async_trait::async_trait]
pub trait DatabaseDriver: SqlDialect + Send + Sync {
    // =======================
    // =   Query Execution   =
    // =======================

    /// Thực thi SQL query bất kỳ (DDL, DML, DQL).
    ///
    /// Driver tự phân biệt loại query và trả về `QueryResult` phù hợp:
    /// - DQL → `QueryResult::Query` (columns + rows)
    /// - DML/DDL → `QueryResult::Execution` (rows_affected, last_insert_rowid)
    async fn execute(&self, query: &str) -> Result<QueryResult, EngineError>;

    /// Kiểm tra kết nối còn sống không.
    ///
    /// Mặc định thực thi `SELECT 1` hoặc equivalent.
    /// Trả về `Ok(())` nếu kết nối OK, `Err` nếu không.
    async fn ping(&self) -> Result<(), EngineError>;

    /// Tên loại database (ví dụ: "SQLite", "PostgreSQL").
    ///
    /// Dùng cho logging, display, debug — không dùng cho logic.
    fn database_type(&self) -> &'static str;

    /// Sinh SQL script từ data-changeset (DML).
    fn generate_changeset_script(&self, changeset: &DataChangeset) -> String {
        let mut script = String::new();

        // Xử lý UPDATEs
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

        // Xử lý DELETEs
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
    ///
    /// Trả về `Vec<TableBrief>` chứa name và kind (Table/View/System).
    /// Dùng cho sidebar listing — nhẹ, nhanh.
    async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError>;

    /// Liệt kê tất cả views trong database.
    ///
    /// Tách riêng khỏi `list_tables` vì sidebar có thể hiển thị views ở section riêng.
    async fn list_views(&self) -> Result<Vec<String>, EngineError>;

    /// Lấy thông tin chi tiết của một table.
    ///
    /// Trả về `TableInfo` chứa columns, primary key, foreign keys, và indexes.
    /// Dùng khi user click vào table trên sidebar.
    ///
    /// # Errors
    ///
    /// Trả về `EngineError::Schema` nếu table không tồn tại.
    async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError>;

    /// Đếm số dòng trong table.
    ///
    /// Dùng `SELECT COUNT(*)` — có thể chậm với table lớn.
    /// Dùng cho sidebar hiển thị "N rows" bên cạnh tên table.
    ///
    /// # Errors
    ///
    /// Trả về `EngineError::Schema` nếu table không tồn tại.
    async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError>;
}

/// Đây là **entry point duy nhất** để tạo driver mới.
///
/// # Thêm database mới
///
/// Thêm match arm tại đây:
/// ```ignore
/// DatabaseKind::Postgres => {
///     let driver = PostgresDriver::new(config).await?;
///     Ok(Box::new(driver))
/// }
/// ```
pub async fn create(config: &DatabaseConfig) -> Result<Box<dyn DatabaseDriver>, EngineError> {
    match config.kind {
        DatabaseKind::Sqlite => {
            let driver = sqlite::SqliteDriver::new(config).await?;
            Ok(Box::new(driver))
        }
    }
}
