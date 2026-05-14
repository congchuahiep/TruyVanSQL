use sqlx::postgres::PgPoolOptions;
use sqlx::{Column as SqlxColumn, Row, TypeInfo};

use crate::DatabaseConfig;
use crate::driver::{DatabaseDriver, SqlDialect};
use crate::error::EngineError;
use crate::result::{Column, QueryResult, Row as ResultRow, Value};
use crate::schema::{
    ColumnInfo, ForeignKeyInfo, IndexInfo, PrimaryKey, TableBrief, TableInfo, TableKind,
};
use crate::{DatabaseKind, NetworkDbConfig};

/// PostgreSQL driver implementation.
///
/// Sử dụng `sqlx::PgPool` bên trong để quản lý connection pool.
pub struct PostgresDriver {
    pool: sqlx::postgres::PgPool,
}

impl PostgresDriver {
    /// Tạo PostgreSQL driver mới từ config.
    ///
    /// Kết nối tới PostgreSQL server và tạo connection pool.
    pub async fn new(config: &DatabaseConfig) -> Result<Self, EngineError> {
        let pg_config = match config {
            DatabaseConfig::Network(c) if c.kind == DatabaseKind::Postgres => c,
            _ => {
                return Err(EngineError::Connection(
                    "Invalid config type for PostgresDriver".into(),
                ));
            }
        };
        Self::from_config(pg_config).await
    }

    async fn from_config(config: &NetworkDbConfig) -> Result<Self, EngineError> {
        let url = config.connection_url();
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections as u32)
            .connect(&url)
            .await
            .map_err(|e| EngineError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

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

    async fn execute_dml(&self, query: &str) -> Result<QueryResult, EngineError> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| EngineError::QueryExecution(e.to_string()))?;

        Ok(QueryResult::Execution {
            rows_affected: result.rows_affected(),
            last_insert_rowid: None,
        })
    }
}

impl SqlDialect for PostgresDriver {
    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier)
    }

    fn format_value(&self, value: &str, data_type: &str) -> String {
        if value == "NULL" {
            return "NULL".into();
        }
        let dt = data_type.to_uppercase();
        if dt.contains("INT")
            || dt.contains("SERIAL")
            || dt.contains("REAL")
            || dt.contains("FLOAT")
            || dt.contains("DOUBLE")
            || dt.contains("NUMERIC")
            || dt.contains("DECIMAL")
        {
            if value
                .chars()
                .all(|c| c.is_digit(10) || c == '.' || c == '-')
            {
                return value.into();
            }
        }
        if dt == "BOOLEAN" || dt == "BOOL" {
            return value.into();
        }
        format!("'{}'", value.replace("'", "''"))
    }
}

#[async_trait::async_trait]
impl DatabaseDriver for PostgresDriver {
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
        "PostgreSQL"
    }

    async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError> {
        let rows = sqlx::query(
            "SELECT tablename, 'TABLE' AS obj_type FROM pg_tables WHERE schemaname = 'public' \
             UNION ALL \
             SELECT viewname, 'VIEW' AS obj_type FROM pg_views WHERE schemaname = 'public' \
             ORDER BY obj_type, tablename",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let tables: Vec<TableBrief> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("tablename").unwrap_or_default();
                let obj_type: String = row.try_get("obj_type").unwrap_or_default();
                let kind = if obj_type == "VIEW" {
                    TableKind::View
                } else {
                    TableKind::Table
                };
                TableBrief { name, kind }
            })
            .collect();

        Ok(tables)
    }

    async fn list_views(&self) -> Result<Vec<String>, EngineError> {
        let rows = sqlx::query(
            "SELECT viewname FROM pg_views WHERE schemaname = 'public' ORDER BY viewname",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let views: Vec<String> = rows
            .iter()
            .filter_map(|row| row.try_get::<String, _>("viewname").ok())
            .collect();

        Ok(views)
    }

    async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, EngineError> {
        if table_name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            return Err(EngineError::Schema(format!(
                "Tên table không hợp lệ: '{table_name}'"
            )));
        }

        let exists: Option<String> = sqlx::query_scalar(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename = $1 \
             UNION ALL \
             SELECT viewname FROM pg_views WHERE schemaname = 'public' AND viewname = $1",
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
        if table_name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            return Err(EngineError::Schema(format!(
                "Tên table không hợp lệ: '{table_name}'"
            )));
        }

        let query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EngineError::Schema(e.to_string()))?;

        Ok(count)
    }
}

impl PostgresDriver {
    async fn get_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, EngineError> {
        let rows = sqlx::query(
            "SELECT column_name, data_type, is_nullable, column_default \
             FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = $1 \
             ORDER BY ordinal_position",
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let pk_columns = self.get_pk_columns(table_name).await?;

        let columns: Vec<ColumnInfo> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("column_name").unwrap_or_default();
                let data_type: String = row.try_get("data_type").unwrap_or_default();
                let is_nullable: String =
                    row.try_get("is_nullable").unwrap_or_else(|_| "YES".into());
                let default_value: Option<String> = row
                    .try_get::<Option<String>, _>("column_default")
                    .unwrap_or(None);

                let is_primary_key = pk_columns.contains(&name);
                ColumnInfo {
                    name,
                    data_type,
                    nullable: is_nullable == "YES",
                    default_value,
                    is_primary_key,
                }
            })
            .collect();

        Ok(columns)
    }

    async fn get_pk_columns(&self, table_name: &str) -> Result<Vec<String>, EngineError> {
        let rows = sqlx::query(
            "SELECT kcu.column_name \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
               AND tc.table_schema = kcu.table_schema \
             WHERE tc.constraint_type = 'PRIMARY KEY' \
               AND tc.table_schema = 'public' \
               AND tc.table_name = $1 \
             ORDER BY kcu.ordinal_position",
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        Ok(rows
            .iter()
            .filter_map(|row| row.try_get::<String, _>("column_name").ok())
            .collect())
    }

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

    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKeyInfo>, EngineError> {
        let rows = sqlx::query(
            "SELECT \
                tc.constraint_name, \
                kcu.column_name, \
                ccu.table_name AS referenced_table, \
                ccu.column_name AS referenced_column, \
                rc.update_rule, \
                rc.delete_rule \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
               AND tc.table_schema = kcu.table_schema \
             JOIN information_schema.constraint_column_usage ccu \
               ON tc.constraint_name = ccu.constraint_name \
               AND tc.table_schema = ccu.table_schema \
             JOIN information_schema.referential_constraints rc \
               ON tc.constraint_name = rc.constraint_name \
             WHERE tc.constraint_type = 'FOREIGN KEY' \
               AND tc.table_schema = 'public' \
               AND tc.table_name = $1 \
             ORDER BY kcu.ordinal_position",
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let mut fk_map: std::collections::HashMap<
            String,
            (
                Vec<String>,
                String,
                Vec<String>,
                Option<String>,
                Option<String>,
            ),
        > = std::collections::HashMap::new();

        for row in &rows {
            let constraint_name: String = row.try_get("constraint_name").unwrap_or_default();
            let column_name: String = row.try_get("column_name").unwrap_or_default();
            let ref_table: String = row.try_get("referenced_table").unwrap_or_default();
            let ref_column: String = row.try_get("referenced_column").unwrap_or_default();
            let on_update: Option<String> = row.try_get("update_rule").ok().flatten();
            let on_delete: Option<String> = row.try_get("delete_rule").ok().flatten();

            let entry = fk_map
                .entry(constraint_name)
                .or_insert_with(|| (Vec::new(), ref_table, Vec::new(), on_update, on_delete));
            entry.0.push(column_name);
            entry.2.push(ref_column);
        }

        let foreign_keys: Vec<ForeignKeyInfo> = fk_map
            .into_values()
            .map(
                |(columns, references_table, references_columns, on_update, on_delete)| {
                    ForeignKeyInfo {
                        name: None,
                        columns,
                        references_table,
                        references_columns,
                        on_delete,
                        on_update,
                    }
                },
            )
            .collect();

        Ok(foreign_keys)
    }

    async fn get_indexes(&self, table_name: &str) -> Result<Vec<IndexInfo>, EngineError> {
        let rows = sqlx::query(
            "SELECT indexname, indexdef \
             FROM pg_indexes \
             WHERE schemaname = 'public' AND tablename = $1",
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EngineError::Schema(e.to_string()))?;

        let mut indexes = Vec::new();

        for row in &rows {
            let name: String = row.try_get("indexname").unwrap_or_default();
            let indexdef: String = row.try_get("indexdef").unwrap_or_default();

            let is_unique = indexdef.to_uppercase().contains("UNIQUE");

            let columns = parse_index_columns(&indexdef);

            indexes.push(IndexInfo {
                name,
                columns,
                is_unique,
            });
        }

        Ok(indexes)
    }
}

/// Parse column names từ PostgreSQL index definition.
///
/// Ví dụ: `CREATE UNIQUE INDEX idx_email ON users (email, name)`
/// → `["email", "name"]`
fn parse_index_columns(indexdef: &str) -> Vec<String> {
    let start = match indexdef.find('(') {
        Some(i) => i + 1,
        None => return Vec::new(),
    };
    let end = match indexdef.rfind(')') {
        Some(i) => i,
        None => return Vec::new(),
    };
    if start >= end {
        return Vec::new();
    }
    indexdef[start..end]
        .split(',')
        .map(|col| col.trim().trim_matches('"').to_string())
        .collect()
}

/// Kiểm tra query có phải DQL không dựa trên keyword đầu tiên.
fn is_dql(query: &str) -> bool {
    let upper = query.trim_start().to_uppercase();
    upper.starts_with("SELECT")
        || upper.starts_with("EXPLAIN")
        || upper.starts_with("WITH")
        || upper.starts_with("SHOW")
        || upper.starts_with("TABLE")
        || upper.starts_with("DESCRIBE")
}

/// Convert một `sqlx::postgres::PgRow` thành `result::Row`.
fn convert_row(row: &sqlx::postgres::PgRow, num_columns: usize) -> ResultRow {
    let values: Vec<Option<Value>> = (0..num_columns).map(|i| convert_value(row, i)).collect();
    ResultRow { values }
}

/// Convert một cell thành `Option<Value>`.
///
/// Thử theo thứ tự: bool → i64 → f64 → String → Vec<u8> → None
fn convert_value(row: &sqlx::postgres::PgRow, idx: usize) -> Option<Value> {
    if let Ok(Some(v)) = row.try_get::<Option<bool>, _>(idx) {
        return Some(Value::Integer(if v { 1 } else { 0 }));
    }
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
