use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Column as SqlxColumn, Row, TypeInfo};

use crate::config::DatabaseConfig;
use crate::driver::{DatabaseDriver, SqlDialect};
use crate::error::EngineError;
use crate::result::{Column, QueryResult, Row as ResultRow, Value};
use crate::schema::{
    ColumnInfo, ForeignKeyInfo, IndexInfo, PrimaryKey, TableBrief, TableInfo, TableKind,
};

/// SQLite driver implementation.
///
/// Sử dụng `sqlx::SqlitePool` bên trong để quản lý connection pool.
/// Mọi query đều đi qua pool — không có single connection.
pub struct SqliteDriver {
    pool: SqlitePool,
}

impl SqliteDriver {
    /// Tạo SQLite driver mới từ config.
    ///
    /// Tự động tạo connection pool với số lượng max connections được chỉ định.
    /// Hỗ trợ:
    /// - File database: `"sqlite:./data.db"`
    /// - In-memory database: `"sqlite::memory:"`
    pub async fn new(config: &DatabaseConfig) -> Result<Self, EngineError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .map_err(|e| EngineError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Thực thi DQL query (SELECT, PRAGMA, EXPLAIN, WITH).
    ///
    /// Trả về `QueryResult::Query` chứa columns và rows.
    async fn execute_dql(&self, query: &str) -> Result<QueryResult, EngineError> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EngineError::QueryExecution(e.to_string()))?;

        if rows.is_empty() {
            return Ok(QueryResult::Query {
                columns: vec![],
                rows: vec![],
            });
        }

        let columns: Vec<Column> = rows[0]
            .columns()
            .iter()
            .map(|col| Column {
                name: col.name().to_string(),
                declared_type: Some(col.type_info().name().to_string()),
            })
            .collect();

        let result_rows: Vec<ResultRow> = rows
            .iter()
            .map(|row| convert_row(row, columns.len()))
            .collect();

        Ok(QueryResult::Query {
            columns,
            rows: result_rows,
        })
    }

    /// Thực thi DML/DDL query (INSERT, UPDATE, DELETE, CREATE, DROP, ...).
    ///
    /// Trả về `QueryResult::Execution` chứa rows_affected và last_insert_rowid.
    async fn execute_dml(&self, query: &str) -> Result<QueryResult, EngineError> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| EngineError::QueryExecution(e.to_string()))?;

        Ok(QueryResult::Execution {
            rows_affected: result.rows_affected(),
            last_insert_rowid: Some(result.last_insert_rowid()),
        })
    }

    /// Lấy danh sách columns của table qua PRAGMA table_info.
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, EngineError> {
        let query = format!("PRAGMA table_info('{table_name}')");
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        let columns: Vec<ColumnInfo> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let data_type: String = row.try_get("type").unwrap_or_default();
                let not_null: bool = row.try_get::<bool, _>("notnull").unwrap_or(false);
                let default_value: Option<String> = row.try_get("dflt_value").ok().flatten();
                let pk: bool = row.try_get::<bool, _>("pk").unwrap_or(false);

                ColumnInfo {
                    name,
                    data_type,
                    nullable: !not_null,
                    default_value,
                    is_primary_key: pk,
                }
            })
            .collect();

        Ok(columns)
    }

    /// Lấy primary key columns từ PRAGMA table_info.
    fn extract_primary_key(columns: &[ColumnInfo]) -> PrimaryKey {
        let pk_columns: Vec<String> = columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.clone())
            .collect();

        PrimaryKey {
            columns: pk_columns,
        }
    }

    /// Lấy foreign keys qua PRAGMA foreign_key_list.
    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKeyInfo>, EngineError> {
        let query = format!("PRAGMA foreign_key_list('{table_name}')");
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        // PRAGMA foreign_key_list trả về:
        // id | seq | table | from | to | on_update | on_delete | match
        // Mỗi FK có thể có nhiều rows (nhiều columns) — group by id
        let mut fk_map: std::collections::HashMap<
            i64,
            (
                String,
                Vec<String>,
                Vec<String>,
                Option<String>,
                Option<String>,
            ),
        > = std::collections::HashMap::new();

        for row in &rows {
            let id: i64 = row.try_get("id").unwrap_or(0);
            let ref_table: String = row.try_get("table").unwrap_or_default();
            let from_col: String = row.try_get("from").unwrap_or_default();
            let to_col: String = row.try_get("to").unwrap_or_default();
            let on_update: Option<String> = row.try_get("on_update").ok().flatten();
            let on_delete: Option<String> = row.try_get("on_delete").ok().flatten();

            let entry = fk_map.entry(id).or_insert_with(|| {
                (
                    ref_table.clone(),
                    Vec::new(),
                    Vec::new(),
                    on_update.clone(),
                    on_delete.clone(),
                )
            });
            entry.1.push(from_col);
            entry.2.push(to_col);
        }

        let foreign_keys: Vec<ForeignKeyInfo> = fk_map
            .into_values()
            .map(|(ref_table, columns, ref_columns, on_update, on_delete)| {
                ForeignKeyInfo {
                    name: None, // SQLite FK không có tên riêng
                    columns,
                    references_table: ref_table,
                    references_columns: ref_columns,
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(foreign_keys)
    }

    /// Lấy indexes qua PRAGMA index_list + PRAGMA index_info.
    async fn get_indexes(&self, table_name: &str) -> Result<Vec<IndexInfo>, EngineError> {
        let query = format!("PRAGMA index_list('{table_name}')");
        let index_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        let mut indexes = Vec::new();

        for index_row in &index_rows {
            let name: String = index_row.try_get("name").unwrap_or_default();
            let unique: bool = index_row.try_get::<bool, _>("unique").unwrap_or(false);

            // Lấy columns của index
            let info_query = format!("PRAGMA index_info('{name}')");
            let info_rows = sqlx::query(&info_query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| EngineError::Schema(e.to_string()))?;

            let columns: Vec<String> = info_rows
                .iter()
                .filter_map(|r| r.try_get::<String, _>("name").ok())
                .collect();

            indexes.push(IndexInfo {
                name,
                columns,
                is_unique: unique,
            });
        }

        Ok(indexes)
    }
}

impl SqlDialect for SqliteDriver {
    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier)
    }

    fn format_value(&self, value: &str, data_type: &str) -> String {
        if value == "NULL" {
            "NULL".into()
        } else {
            let dt = data_type.to_uppercase();
            if (dt.contains("INT") || dt.contains("REAL") || dt.contains("FLOAT"))
                && value
                    .chars()
                    .all(|c| c.is_digit(10) || c == '.' || c == '-')
            {
                value.into()
            } else {
                format!("'{}'", value.replace("'", "''"))
            }
        }
    }
}
#[async_trait::async_trait]
impl DatabaseDriver for SqliteDriver {
    async fn execute(&self, query: &str) -> Result<QueryResult, EngineError> {
        let trimmed = query.trim();

        if is_dql(trimmed) {
            self.execute_dql(query).await
        } else {
            self.execute_dml(query).await
        }
    }

    async fn ping(&self) -> Result<(), EngineError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| EngineError::Connection(e.to_string()))?;
        Ok(())
    }

    fn database_type(&self) -> &'static str {
        "SQLite"
    }

    async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError> {
        let rows = sqlx::query(
            "SELECT name, type FROM sqlite_master
             WHERE type IN ('table', 'view')
             ORDER BY type, name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let tables: Vec<TableBrief> = rows
            .iter()
            .filter_map(|row| {
                let name: String = row.try_get("name").ok()?;
                let obj_type: String = row.try_get("type").ok()?;

                // Bỏ qua system tables (bắt đầu bằng sqlite_)
                if name.starts_with("sqlite_") {
                    return None;
                }

                let kind = match obj_type.as_str() {
                    "view" => TableKind::View,
                    _ => TableKind::Table,
                };

                Some(TableBrief { name, kind })
            })
            .collect();

        Ok(tables)
    }

    async fn list_views(&self) -> Result<Vec<String>, EngineError> {
        let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'view' ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        let views: Vec<String> = rows
            .iter()
            .filter_map(|row| row.try_get::<String, _>("name").ok())
            .collect();

        Ok(views)
    }

    async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError> {
        // Kiểm tra table có tồn tại không
        let exists = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?",
        )
        .bind(table_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        if exists.is_none() {
            return Err(EngineError::Schema(format!(
                "Table '{table_name}' không tồn tại"
            )));
        }

        let columns = self.get_columns(table_name).await?;
        let primary_key = Self::extract_primary_key(&columns);
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

    async fn get_table_row_count(&self, table_name: &str) -> Result<i64, EngineError> {
        // Validate table name to prevent SQL injection
        if table_name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            return Err(EngineError::Schema(format!(
                "Tên table không hợp lệ: '{table_name}'"
            )));
        }

        let query = format!("SELECT COUNT(*) FROM \"{table_name}\"");
        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        Ok(count)
    }
}

/// Kiểm tra query có phải DQL không dựa trên keyword đầu tiên.
fn is_dql(query: &str) -> bool {
    let upper = query.trim_start().to_uppercase();
    upper.starts_with("SELECT")
        || upper.starts_with("PRAGMA")
        || upper.starts_with("EXPLAIN")
        || upper.starts_with("WITH")
}

/// Convert một `sqlx::SqliteRow` thành `result::Row`.
fn convert_row(row: &sqlx::sqlite::SqliteRow, num_columns: usize) -> ResultRow {
    let values: Vec<Option<Value>> = (0..num_columns).map(|i| convert_value(row, i)).collect();

    ResultRow { values }
}

/// Convert một cell thành `Option<Value>`.
fn convert_value(row: &sqlx::sqlite::SqliteRow, idx: usize) -> Option<Value> {
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(idx) {
        return Some(Value::Integer(v));
    }
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(idx) {
        return Some(Value::Float(v));
    }
    if let Ok(Some(v)) = row.try_get::<Option<String>, _>(idx) {
        return Some(Value::Text(v));
    }
    if let Ok(Some(v)) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        return Some(Value::Blob(v));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DataChangeset,
        schema::{ColumnData, RowDelete, RowUpdate},
    };

    async fn create_driver() -> SqliteDriver {
        let config = DatabaseConfig::sqlite("sqlite::memory:");
        SqliteDriver::new(&config).await.unwrap()
    }

    // ===== is_dql tests =====

    #[test]
    fn test_is_dql_select() {
        assert!(is_dql("SELECT * FROM users"));
        assert!(is_dql("  SELECT * FROM users"));
        assert!(is_dql("select * from users"));
    }

    #[test]
    fn test_is_dql_pragma() {
        assert!(is_dql("PRAGMA table_info(users)"));
    }

    #[test]
    fn test_is_dql_explain() {
        assert!(is_dql("EXPLAIN SELECT * FROM users"));
    }

    #[test]
    fn test_is_dql_with() {
        assert!(is_dql("WITH cte AS (SELECT 1) SELECT * FROM cte"));
    }

    #[test]
    fn test_is_not_dql() {
        assert!(!is_dql("INSERT INTO users VALUES (1, 'Alice')"));
        assert!(!is_dql("CREATE TABLE users (id INTEGER)"));
    }

    // ===== Execution tests =====

    #[tokio::test]
    async fn test_database_type() {
        let driver = create_driver().await;
        assert_eq!(driver.database_type(), "SQLite");
    }

    #[tokio::test]
    async fn test_ping() {
        let driver = create_driver().await;
        assert!(driver.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_create_table() {
        let driver = create_driver().await;
        let result = driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();
        match result {
            QueryResult::Execution { rows_affected, .. } => assert_eq!(rows_affected, 0),
            _ => panic!("Expected Execution"),
        }
    }

    #[tokio::test]
    async fn test_execute_insert() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();

        let result = driver
            .execute("INSERT INTO test (name) VALUES ('Alice')")
            .await
            .unwrap();
        match result {
            QueryResult::Execution {
                rows_affected,
                last_insert_rowid,
            } => {
                assert_eq!(rows_affected, 1);
                assert_eq!(last_insert_rowid, Some(1));
            }
            _ => panic!("Expected Execution"),
        }
    }

    #[tokio::test]
    async fn test_execute_select() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();
        driver
            .execute("INSERT INTO test (name) VALUES ('Alice'), ('Bob')")
            .await
            .unwrap();

        let result = driver
            .execute("SELECT * FROM test ORDER BY id")
            .await
            .unwrap();
        match result {
            QueryResult::Query { columns, rows } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected Query"),
        }
    }

    #[tokio::test]
    async fn test_execute_select_with_null() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER, val TEXT)")
            .await
            .unwrap();
        driver
            .execute("INSERT INTO test VALUES (1, NULL)")
            .await
            .unwrap();

        let result = driver.execute("SELECT * FROM test").await.unwrap();
        match result {
            QueryResult::Query { rows, .. } => {
                assert_eq!(rows[0].values[1], None);
            }
            _ => panic!("Expected Query"),
        }
    }

    #[tokio::test]
    async fn test_execute_invalid_query() {
        let driver = create_driver().await;
        assert!(driver.execute("INVALID SQL").await.is_err());
    }

    // ===== Schema Introspection tests =====

    #[tokio::test]
    async fn test_list_tables_empty() {
        let driver = create_driver().await;
        let tables = driver.list_tables().await.unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_list_tables_multiple() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER)")
            .await
            .unwrap();
        driver
            .execute("CREATE TABLE posts (id INTEGER)")
            .await
            .unwrap();
        driver
            .execute("CREATE TABLE comments (id INTEGER)")
            .await
            .unwrap();

        let tables = driver.list_tables().await.unwrap();
        assert_eq!(tables.len(), 3);
        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"users"));
        assert!(names.contains(&"posts"));
        assert!(names.contains(&"comments"));
        // Tất cả đều là Table kind
        assert!(tables.iter().all(|t| t.kind == TableKind::Table));
    }

    #[tokio::test]
    async fn test_list_tables_includes_views() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .await
            .unwrap();
        driver
            .execute("CREATE VIEW user_names AS SELECT name FROM users")
            .await
            .unwrap();

        let tables = driver.list_tables().await.unwrap();
        assert_eq!(tables.len(), 2);

        let view = tables.iter().find(|t| t.kind == TableKind::View).unwrap();
        assert_eq!(view.name, "user_names");

        let table = tables.iter().find(|t| t.kind == TableKind::Table).unwrap();
        assert_eq!(table.name, "users");
    }

    #[tokio::test]
    async fn test_list_tables_ignores_system() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER)")
            .await
            .unwrap();

        // sqlite_sequence được tạo tự động bởi AUTOINCREMENT, nhưng không nên show
        driver
            .execute("CREATE TABLE test_auto (id INTEGER PRIMARY KEY AUTOINCREMENT)")
            .await
            .unwrap();

        let tables = driver.list_tables().await.unwrap();
        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
        // sqlite_sequence nếu có thì không nên nằm trong danh sách
        assert!(!names.iter().any(|n| n.starts_with("sqlite_")));
    }

    #[tokio::test]
    async fn test_list_views_empty() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER)")
            .await
            .unwrap();

        let views = driver.list_views().await.unwrap();
        assert!(views.is_empty());
    }

    #[tokio::test]
    async fn test_list_views_with_views() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .await
            .unwrap();
        driver
            .execute("CREATE VIEW active_users AS SELECT * FROM users WHERE id > 0")
            .await
            .unwrap();
        driver
            .execute("CREATE VIEW user_names AS SELECT name FROM users")
            .await
            .unwrap();

        let views = driver.list_views().await.unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.contains(&"active_users".to_string()));
        assert!(views.contains(&"user_names".to_string()));
    }

    #[tokio::test]
    async fn test_get_table_info_columns() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age REAL)")
            .await
            .unwrap();

        let info = driver.get_table_info("test").await.unwrap();
        assert_eq!(info.name, "test");
        assert_eq!(info.columns.len(), 3);

        // id column
        let id_col = &info.columns[0];
        assert_eq!(id_col.name, "id");
        assert_eq!(id_col.data_type, "INTEGER");
        assert!(id_col.is_primary_key);

        // name column
        let name_col = &info.columns[1];
        assert_eq!(name_col.name, "name");
        assert_eq!(name_col.data_type, "TEXT");
        assert!(!name_col.nullable); // NOT NULL

        // age column
        let age_col = &info.columns[2];
        assert_eq!(age_col.name, "age");
        assert_eq!(age_col.data_type, "REAL");
        assert!(age_col.nullable); // nullable by default
    }

    #[tokio::test]
    async fn test_get_table_info_composite_pk() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (a INTEGER, b INTEGER, c TEXT, PRIMARY KEY (a, b))")
            .await
            .unwrap();

        let info = driver.get_table_info("test").await.unwrap();
        assert_eq!(info.primary_key.columns.len(), 2);
        assert!(info.primary_key.columns.contains(&"a".to_string()));
        assert!(info.primary_key.columns.contains(&"b".to_string()));
    }

    #[tokio::test]
    async fn test_get_table_info_default_value() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER, status TEXT DEFAULT 'active')")
            .await
            .unwrap();

        let info = driver.get_table_info("test").await.unwrap();
        let status_col = &info.columns[1];
        assert_eq!(status_col.name, "status");
        assert_eq!(status_col.default_value, Some("'active'".to_string()));
    }

    #[tokio::test]
    async fn test_get_table_info_foreign_keys() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();
        driver
            .execute("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)")
            .await
            .unwrap();

        let info = driver.get_table_info("posts").await.unwrap();
        assert_eq!(info.foreign_keys.len(), 1);

        let fk = &info.foreign_keys[0];
        assert_eq!(fk.columns, vec!["user_id"]);
        assert_eq!(fk.references_table, "users");
        assert_eq!(fk.references_columns, vec!["id"]);
        assert_eq!(fk.on_delete, Some("CASCADE".to_string()));
    }

    #[tokio::test]
    async fn test_get_table_info_indexes() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, email TEXT)")
            .await
            .unwrap();
        driver
            .execute("CREATE UNIQUE INDEX idx_email ON test(email)")
            .await
            .unwrap();

        let info = driver.get_table_info("test").await.unwrap();

        // SQLite tự tạo index cho PK + index thủ công
        let email_idx = info.indexes.iter().find(|i| i.name == "idx_email");
        assert!(email_idx.is_some());
        let email_idx = email_idx.unwrap();
        assert!(email_idx.is_unique);
        assert!(email_idx.columns.contains(&"email".to_string()));
    }

    #[tokio::test]
    async fn test_get_table_info_no_indexes() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER, name TEXT)")
            .await
            .unwrap();

        let info = driver.get_table_info("test").await.unwrap();
        assert!(info.indexes.is_empty());
    }

    #[tokio::test]
    async fn test_get_table_info_nonexistent() {
        let driver = create_driver().await;
        let result = driver.get_table_info("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::Schema(msg) => assert!(msg.contains("không tồn tại")),
            _ => panic!("Expected Schema error"),
        }
    }

    #[tokio::test]
    async fn test_get_table_row_count() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();
        driver
            .execute("INSERT INTO test VALUES (1), (2), (3)")
            .await
            .unwrap();

        let count = driver.get_table_row_count("test").await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_get_table_row_count_empty() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER)")
            .await
            .unwrap();

        let count = driver.get_table_row_count("test").await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_table_row_count_nonexistent() {
        let driver = create_driver().await;
        let result = driver.get_table_row_count("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_table_row_count_invalid_name() {
        let driver = create_driver().await;
        let result = driver.get_table_row_count("test; DROP TABLE users").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_inserts_last_rowid() {
        let driver = create_driver().await;
        driver
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)")
            .await
            .unwrap();

        for i in 1..=5 {
            let result = driver
                .execute(&format!("INSERT INTO test (val) VALUES ('item{i}')"))
                .await
                .unwrap();
            match result {
                QueryResult::Execution {
                    last_insert_rowid, ..
                } => {
                    assert_eq!(last_insert_rowid, Some(i));
                }
                _ => panic!("Expected Execution"),
            }
        }
    }

    #[test]
    fn test_generate_changeset_script() {
        let config = DatabaseConfig::sqlite("sqlite::memory:");
        // Cần tokio runtime để tạo driver, nhưng generate_changeset_script không async
        // Tuy nhiên SqliteDriver::new là async.
        // Ta có thể mock hoặc sử dụng block_on nếu cần, nhưng ở đây ta có thể tạo driver đơn giản.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let driver = rt.block_on(async { SqliteDriver::new(&config).await.unwrap() });

        let changeset = DataChangeset {
            table_name: "users".to_string(),
            updates: vec![RowUpdate {
                pk_conditions: vec![ColumnData {
                    column_name: "id".to_string(),
                    value: "1".to_string(),
                    data_type: "INTEGER".to_string(),
                }],
                changes: vec![ColumnData {
                    column_name: "name".to_string(),
                    value: "Alice O'Neil".to_string(),
                    data_type: "TEXT".to_string(),
                }],
            }],
            deletes: vec![RowDelete {
                pk_conditions: vec![ColumnData {
                    column_name: "id".to_string(),
                    value: "2".to_string(),
                    data_type: "INTEGER".to_string(),
                }],
            }],
        };

        let script = driver.generate_changeset_script(&changeset);
        assert_eq!(
            script,
            "UPDATE \"users\" SET \"name\" = 'Alice O''Neil' WHERE \"id\" = 1;\nDELETE FROM \"users\" WHERE \"id\" = 2;\n"
        );
    }
}
