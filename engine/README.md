# Engine — Thư viện lõi

Chứa tất cả logic liên quan đến database: drivers, query execution, schema introspection.

## Modules

```
engine/src/
├── lib.rs                    # Entry point (re-exports)
├── error.rs                  # EngineError
├── sql_client.rs            # SqlClient wrapper
├── database_config/         # DatabaseConfig abstraction
├── driver/                  # Driver implementations
│   ├── mod.rs              # DatabaseDriver trait
│   ├── sqlite.rs           # SqliteDriver
│   └── postgres.rs         # PostgresDriver
├── result/                  # Query result types
│   ├── column.rs
│   ├── row.rs
│   ├── value.rs
│   └── query_result.rs
└── schema/                  # Schema metadata
    ├── table_brief.rs
    ├── table_info.rs
    └── ...
```

## Module tác dụng

| Module | Mô tả |
|--------|-------|
| `database_config/` | DatabaseConfig trait + SQLite/PostgreSQL configs |
| `driver/` | DatabaseDriver trait (SQLite, PostgreSQL implementations) |
| `result/` | Query result types (Column, Row, Value, QueryResult) |
| `schema/` | Schema metadata (TableInfo, ColumnInfo, PrimaryKey, v.v.) |

## Thiết kế

### DatabaseDriver trait

Dynamic dispatch cho phép dễ dàng thêm driver mới:

```rust
pub trait DatabaseDriver: SqlDialect + Send + Sync {
    async fn execute(&self, query: &str) -> Result<QueryResult, EngineError>;
    async fn list_tables(&self) -> Result<Vec<TableBrief>, EngineError>;
    async fn get_table_info(&self, table: &str) -> Result<TableInfo, EngineError>;
    // ...
}
```

### SqlClient wrapper

`SqlClient` bọc `Arc<dyn DatabaseDriver>`, implement `Deref` cho tiện sử dụng.

## Quy ước

- Import flat: `use engine::X;` — KHÔNG nested
- `mod.rs` chỉ re-export, KHÔNG định nghĩa logic
- File name = snake_case của struct chính

## Ví dụ sử dụng

```rust
use engine::{DatabaseConfig, SqlClient};

let config = DatabaseConfig::sqlite("sqlite:./mydata.db");
let client = SqlClient::connect(config).await?;

let tables = client.list_tables().await?;
let result = client.execute("SELECT * FROM users").await?;
```