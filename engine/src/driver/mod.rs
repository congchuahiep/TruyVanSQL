pub mod postgres;
pub mod sqlite;

use crate::database_config::{DatabaseConfig, DatabaseKind};
use crate::error::EngineError;
use crate::result::QueryResult;
use crate::schema::{DataChangeset, DatabaseBrief, SchemaBrief, TableBrief, TableInfo};
use crate::{ColumnInfo, ForeignKeyInfo, IndexInfo, PrimaryKey};

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
    ///
    /// - DQL: [`Self::execute_dql`]
    /// - DML: [`Self::execute_dml`]
    async fn execute(&self, query: &str) -> Result<QueryResult, EngineError> {
        let trimmed = query.trim();

        if self.is_dql(trimmed) {
            self.execute_dql(query).await
        } else {
            self.execute_dml(query).await
        }
    }

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

    /// Kiểm tra query có phải DQL không dựa trên keyword đầu tiên.
    fn is_dql(&self, query: &str) -> bool {
        let upper = query.trim_start().to_uppercase();
        self.dql_keywords().iter().any(|kw| upper.starts_with(kw))
    }

    /// Kiểm tra kết nối còn sống không.
    async fn ping(&self) -> Result<(), EngineError> {
        self.execute("SELECT 1").await?;
        Ok(())
    }

    /// Tên loại database (ví dụ: "SQLite", "PostgreSQL").
    fn database_type(&self) -> &'static str;

    /// Danh sách keyword DQL cho driver này. Sử dụng để xác định query là DQL hay DML.
    /// Mỗi driver override method này để khai báo keyword riêng.
    ///
    /// # Ví dụ
    /// - PostgreSQL: `["SELECT", "EXPLAIN", "WITH", "SHOW", "TABLE", "DESCRIBE"]`
    /// - SQLite: `["SELECT", "PRAGMA", "EXPLAIN", "WITH"]`
    fn dql_keywords(&self) -> &'static [&'static str];

    /// Thực thi DQL query (SELECT, PRAGMA, EXPLAIN, WITH, ...), hàm này được gọi tại hàm
    /// [`Self::execute`] khi query được xác định là DQL.
    async fn execute_dql(&self, query: &str) -> Result<QueryResult, EngineError>;

    /// Thực thi DML/DDL query (INSERT, UPDATE, DELETE, CREATE, DROP, ...). Hàm này được gọi tại hàm
    /// [`Self::execute`] khi query được xác định là DML.
    async fn execute_dml(&self, query: &str) -> Result<QueryResult, EngineError>;

    // ============================
    // =   Schema Introspection   =
    // ============================

    /// Liệt kê tables trong một schema cụ thể.
    async fn list_tables(&self, schema: &str) -> Result<Vec<TableBrief>, EngineError>;

    /// Liệt kê views trong một schema cụ thể.
    async fn list_views(&self, schema: &str) -> Result<Vec<TableBrief>, EngineError>;

    /// Liệt kê tất cả databases trong server (chỉ PostgreSQL).
    /// Với SQLite luôn trả về vec rỗng vì SQLite không có khái niệm nhiều databases.
    async fn list_databases(&self) -> Result<Vec<DatabaseBrief>, EngineError>;

    /// Liệt kê tất cả schemas trong database hiện tại.
    async fn list_schemas(&self) -> Result<Vec<SchemaBrief>, EngineError>;

    /// Kiểm tra table/view có tồn tại không.
    async fn table_exists(&self, table_name: &str) -> Result<bool, EngineError>;

    /// Lấy thông tin chi tiết của một table.
    async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError> {
        self.validate_table_name(table_name)?;
        if !self.table_exists(table_name).await? {
            return Err(EngineError::Schema(format!(
                "Table '{table_name}' không tồn tại"
            )));
        }
        let columns = self.get_columns(table_name).await?;
        let primary_key = extract_primary_key(&columns);
        let foreign_keys = self.get_foreign_keys(table_name).await?;
        let indexes = self.get_indexes(table_name).await?;
        Ok(TableInfo {
            name: table_name.to_string(),
            columns,
            primary_key,
            foreign_keys,
            indexes,
        })
    }

    /// Đếm số dòng trong table.
    async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError>;

    /// Lấy danh sách columns của table.
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, EngineError>;

    /// Lấy foreign keys của table.
    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKeyInfo>, EngineError>;

    /// Lấy indexes của table.
    async fn get_indexes(&self, table_name: &str) -> Result<Vec<IndexInfo>, EngineError>;

    // =========================
    // =        Utils          =
    // =========================

    /// Validate table name, chống SQL injection.
    /// Override method này nếu driver cho phép ký tự đặc biệt
    /// (ví dụ: MSSQL cho phép `[` `]`).
    fn validate_table_name(&self, table_name: &str) -> Result<(), EngineError> {
        if table_name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            return Err(EngineError::Schema(format!(
                "Tên table không hợp lệ: '{table_name}'"
            )));
        }
        Ok(())
    }
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

/// Trích xuất primary key từ danh sách columns.
pub fn extract_primary_key(columns: &[ColumnInfo]) -> PrimaryKey {
    let pk_columns: Vec<String> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.clone())
        .collect();
    PrimaryKey {
        columns: pk_columns,
    }
}
