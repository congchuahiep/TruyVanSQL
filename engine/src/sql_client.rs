use crate::config::DatabaseConfig;
use crate::driver::{self, DatabaseDriver};
use crate::error::EngineError;
use crate::result::QueryResult;
use crate::schema::{TableBrief, TableInfo};

/// Entry point chính của SQL Client.
///
/// `SqlClient` bọc một [`DatabaseDriver`] và cung cấp API đơn giản để:
/// - Kết nối database (qua [`SqlClient::connect`])
/// - Thực thi query (qua [`SqlClient::execute`])
/// - Kiểm tra kết nối (qua [`SqlClient::ping`])
/// - Khám phá schema (qua [`SqlClient::list_tables`], [`SqlClient::get_table_info`])
///
/// # Ví dụ
///
/// ```ignore
/// use engine::{SqlClient, DatabaseConfig};
///
/// let config = DatabaseConfig::sqlite("sqlite:./mydata.db");
/// let client = SqlClient::connect(config).await?;
///
/// // Liệt kê tables
/// let tables = client.list_tables().await?;
/// for t in &tables {
///     println!("{}: {:?}", t.name, t.kind);
/// }
///
/// // Xem chi tiết table
/// let info = client.get_table_info("users").await?;
/// for col in &info.columns {
///     println!("  {} ({})", col.name, col.data_type);
/// }
/// ```
pub struct SqlClient {
    driver: Box<dyn DatabaseDriver>,
}

impl SqlClient {
    /// Tạo SqlClient mới từ config.
    ///
    /// Tự động chọn driver phù hợp dựa trên `config.kind`:
    /// - `DatabaseKind::Sqlite` → `SqliteDriver`
    ///
    /// # Errors
    ///
    /// - `EngineError::Connection` nếu không thể kết nối.
    /// - `EngineError::UnsupportedDatabase` nếu database type chưa có driver.
    pub async fn connect(config: DatabaseConfig) -> Result<Self, EngineError> {
        let driver = driver::create(&config).await?;
        Ok(Self { driver })
    }

    /// Thực thi SQL query bất kỳ (DDL, DML, DQL).
    ///
    /// Tự động phân biệt loại query và trả về `QueryResult` phù hợp.
    ///
    /// # Returns
    ///
    /// - `QueryResult::Execution` cho INSERT, UPDATE, DELETE, CREATE, DROP, ...
    /// - `QueryResult::Query` cho SELECT, PRAGMA, EXPLAIN, WITH ...
    pub async fn execute(&self, query: &str) -> Result<QueryResult, EngineError> {
        self.driver.execute(query).await
    }

    /// Kiểm tra kết nối còn sống không.
    ///
    /// Trả về `Ok(())` nếu kết nối OK, `Err` nếu không.
    pub async fn ping(&self) -> Result<(), EngineError> {
        self.driver.ping().await
    }

    /// Lấy tên loại database đang kết nối (ví dụ: "SQLite", "PostgreSQL").
    pub fn database_type(&self) -> &'static str {
        self.driver.database_type()
    }

    // === Schema Introspection ===

    /// Liệt kê tất cả tables và views trong database.
    ///
    /// Trả về `Vec<TableBrief>` — nhẹ, chỉ chứa name và kind.
    /// Dùng cho sidebar listing.
    pub async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError> {
        self.driver.list_tables().await
    }

    /// Liệt kê tất cả views trong database.
    ///
    /// Tách riêng vì sidebar có thể hiển thị views ở section riêng.
    pub async fn list_views(&self) -> Result<Vec<String>, EngineError> {
        self.driver.list_views().await
    }

    /// Lấy thông tin chi tiết của một table.
    ///
    /// Trả về `TableInfo` chứa columns, primary key, foreign keys, indexes.
    /// Dùng khi user click vào table trên sidebar.
    ///
    /// # Errors
    ///
    /// Trả về `EngineError::Schema` nếu table không tồn tại.
    pub async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError> {
        self.driver.get_table_info(table_name).await
    }

    /// Đếm số dòng trong table.
    ///
    /// Dùng `SELECT COUNT(*)` — có thể chậm với table lớn.
    /// Dùng cho sidebar hiển thị "N rows" bên cạnh tên table.
    ///
    /// # Errors
    ///
    /// Trả về `EngineError::Schema` nếu table không tồn tại.
    pub async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError> {
        self.driver.get_table_row_count(table_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::Value;
    use crate::schema::TableKind;

    fn sqlite_memory_config() -> DatabaseConfig {
        DatabaseConfig::sqlite("sqlite::memory:")
    }

    #[tokio::test]
    async fn test_connect_sqlite_memory() {
        let client = SqlClient::connect(sqlite_memory_config()).await;
        assert!(client.is_ok());
        assert_eq!(client.unwrap().database_type(), "SQLite");
    }

    #[tokio::test]
    async fn test_ping() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        assert!(client.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_create_table() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        let result = client
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await;
        assert!(result.is_ok());
        match result.unwrap() {
            QueryResult::Execution { rows_affected, .. } => assert_eq!(rows_affected, 0),
            _ => panic!("Expected Execution result"),
        }
    }

    #[tokio::test]
    async fn test_execute_insert() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();

        let result = client
            .execute("INSERT INTO test (name) VALUES ('Alice')")
            .await;
        assert!(result.is_ok());
        match result.unwrap() {
            QueryResult::Execution {
                rows_affected,
                last_insert_rowid,
            } => {
                assert_eq!(rows_affected, 1);
                assert_eq!(last_insert_rowid, Some(1));
            }
            _ => panic!("Expected Execution result"),
        }
    }

    #[tokio::test]
    async fn test_execute_select() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();
        client
            .execute("INSERT INTO test (name) VALUES ('Alice'), ('Bob')")
            .await
            .unwrap();

        let result = client.execute("SELECT * FROM test ORDER BY id").await;
        assert!(result.is_ok());
        match result.unwrap() {
            QueryResult::Query { columns, rows } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(columns[0].name, "id");
                assert_eq!(columns[1].name, "name");
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].values[0], Some(Value::Integer(1)));
                assert_eq!(rows[0].values[1], Some(Value::Text("Alice".to_string())));
            }
            _ => panic!("Expected Query result"),
        }
    }

    #[tokio::test]
    async fn test_execute_select_empty() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();

        let result = client.execute("SELECT * FROM test").await;
        assert!(result.is_ok());
        match result.unwrap() {
            QueryResult::Query { columns, rows } => {
                assert!(columns.is_empty());
                assert!(rows.is_empty());
            }
            _ => panic!("Expected Query result"),
        }
    }

    #[tokio::test]
    async fn test_execute_select_with_null() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)")
            .await
            .unwrap();
        client
            .execute("INSERT INTO test (val) VALUES (NULL)")
            .await
            .unwrap();

        let result = client.execute("SELECT * FROM test").await.unwrap();
        match result {
            QueryResult::Query { rows, .. } => {
                assert_eq!(rows[0].values[1], None);
            }
            _ => panic!("Expected Query result"),
        }
    }

    #[tokio::test]
    async fn test_execute_invalid_query() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        let result = client.execute("INVALID SQL QUERY").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_full_flow() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();

        client
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
            .await
            .unwrap();
        client
            .execute("INSERT INTO users (name, age) VALUES ('Alice', 30)")
            .await
            .unwrap();
        client
            .execute("INSERT INTO users (name, age) VALUES ('Bob', 25)")
            .await
            .unwrap();

        let result = client
            .execute("SELECT name, age FROM users WHERE age > 20 ORDER BY age DESC")
            .await
            .unwrap();
        match result {
            QueryResult::Query { columns, rows } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].values[0], Some(Value::Text("Alice".to_string())));
                assert_eq!(rows[0].values[1], Some(Value::Integer(30)));
            }
            _ => panic!("Expected Query result"),
        }

        let update_result = client
            .execute("UPDATE users SET age = 31 WHERE name = 'Alice'")
            .await
            .unwrap();
        match update_result {
            QueryResult::Execution { rows_affected, .. } => assert_eq!(rows_affected, 1),
            _ => panic!("Expected Execution result"),
        }

        let delete_result = client
            .execute("DELETE FROM users WHERE name = 'Bob'")
            .await
            .unwrap();
        match delete_result {
            QueryResult::Execution { rows_affected, .. } => assert_eq!(rows_affected, 1),
            _ => panic!("Expected Execution result"),
        }

        let final_result = client.execute("SELECT * FROM users").await.unwrap();
        match final_result {
            QueryResult::Query { rows, .. } => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].values[1], Some(Value::Text("Alice".to_string())));
                assert_eq!(rows[0].values[2], Some(Value::Integer(31)));
            }
            _ => panic!("Expected Query result"),
        }
    }

    // ===== Schema Introspection tests (via SqlClient) =====

    #[tokio::test]
    async fn test_list_tables() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE users (id INTEGER)")
            .await
            .unwrap();
        client
            .execute("CREATE TABLE posts (id INTEGER)")
            .await
            .unwrap();

        let tables = client.list_tables().await.unwrap();
        assert_eq!(tables.len(), 2);
        assert!(tables.iter().any(|t| t.name == "users"));
        assert!(tables.iter().any(|t| t.name == "posts"));
    }

    #[tokio::test]
    async fn test_list_tables_empty() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        let tables = client.list_tables().await.unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_list_tables_with_views() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .await
            .unwrap();
        client
            .execute("CREATE VIEW user_names AS SELECT name FROM users")
            .await
            .unwrap();

        let tables = client.list_tables().await.unwrap();
        assert_eq!(tables.len(), 2);
        assert!(tables.iter().any(|t| t.kind == TableKind::View));
        assert!(tables.iter().any(|t| t.kind == TableKind::Table));
    }

    #[tokio::test]
    async fn test_list_views() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .await
            .unwrap();
        client
            .execute("CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0")
            .await
            .unwrap();

        let views = client.list_views().await.unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0], "active_users");
    }

    #[tokio::test]
    async fn test_get_table_info() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age REAL)")
            .await
            .unwrap();

        let info = client.get_table_info("users").await.unwrap();
        assert_eq!(info.name, "users");
        assert_eq!(info.columns.len(), 3);
        assert_eq!(info.columns[0].name, "id");
        assert!(info.columns[0].is_primary_key);
        assert!(!info.columns[1].nullable);
        assert_eq!(info.primary_key.columns, vec!["id"]);
    }

    #[tokio::test]
    async fn test_get_table_info_nonexistent() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        let result = client.get_table_info("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_table_row_count() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER)")
            .await
            .unwrap();
        client
            .execute("INSERT INTO test VALUES (1), (2), (3)")
            .await
            .unwrap();

        let count = client.get_table_row_count("test").await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_get_table_row_count_empty() {
        let client = SqlClient::connect(sqlite_memory_config()).await.unwrap();
        client
            .execute("CREATE TABLE test (id INTEGER)")
            .await
            .unwrap();

        let count = client.get_table_row_count("test").await.unwrap();
        assert_eq!(count, 0);
    }
}
