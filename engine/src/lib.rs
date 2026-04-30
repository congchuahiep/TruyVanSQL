pub mod config;
pub mod driver;
pub mod error;
pub mod result;
pub mod schema;
pub mod sql_client;

// Re-exports — API public gọn gàng, caller không cần biết cấu trúc module bên trong.
pub use config::{DatabaseConfig, DatabaseKind};
pub use error::EngineError;
pub use result::{Column, QueryResult, Row, Value};
pub use schema::{
    ColumnInfo, ForeignKeyInfo, IndexInfo, PrimaryKey, TableBrief, TableInfo, TableKind,
};
pub use sql_client::SqlClient;
