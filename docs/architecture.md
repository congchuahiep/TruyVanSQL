# Kiến trúc TruyVanSQL

## Tổng quan

TruyVanSQL là ứng dụng SQL client đa nền tảng viết bằng Rust, sử dụng GPUI framework (Zed Editor) cho UI và sqlx cho database operations.

## Kiến trúc hệ thống

```
User Input (GPUI)
    ↓
Workspace (Root View)
    ↓
Panel (Sidebar, TabBar, TabContent)
    ↓
ConnectionStore → DatabaseConnection → SqlClient
                                   ↓
                             DatabaseDriver (sqlx)
```

## Các thành phần chính

### Engine — Thư viện lõi

Chứa tất cả logic liên quan đến database, **không import GPUI**.

- **database_config/** — DatabaseConfig trait + implementations (SQLite, PostgreSQL)
- **driver/** — Driver abstractions (DatabaseDriver trait, SqliteDriver, PostgresDriver)
- **result/** — Query result types (Column, Row, Value, QueryResult)
- **schema/** — Schema metadata (TableBrief, TableInfo, ColumnInfo, v.v.)

### Desktop — Ứng dụng GUI

Chứa UI với GPUI framework. Cấu trúc feature-based, không phải layer-based.

- **connection/** — Connection management (DatabaseConnection entity, ConnectionStore)
- **panel/** — UI panels (Sidebar, Titlebar, Tab infrastructure, TabContent)
- **workspace/** — Root view điều phối các panel
- **shared/** — Shared components (SmartDataGrid)
- **window/** — Secondary windows (Connection dialog)
- **theme/** — Theme system với Mica backdrop support

### Assets — Icons

SVG icons embed vào binary sử dụng RustEmbed.

### CLI — Command-line tool (WIP)

## Thiết kế quan trọng

### 1. Engine là library thuần

Engine crate **không import GPUI**. Import flat:
```rust
use engine::DatabaseConfig;  // ✅
use engine::sql_client::SqlClient;  // ✅
```

### 2. Driver abstraction

`DatabaseDriver` trait cho phép dynamic dispatch (`dyn DatabaseDriver`), dễ thêm database mới.

### 3. Polymorphic Tabs

"Everything is a Tab Item" — SQL Editor, Table Viewer đều implement `TabItem` trait, quản lý bởi `TabManager`.

### 4. Custom Titlebar

Ẩn OS titlebar, tự vẽ titlebar bằng GPUI với window controls. Hỗ trợ Mica backdrop trên Windows.

## Công nghệ

| Thành phần | Công nghệ |
|-----------|----------|
| UI | GPUI (Zed), gpui-component |
| Database | sqlx (SQLite + PostgreSQL) |
| Async | tokio |
| Icons | RustEmbed |

## Quy ước

- **Engine**: Import flat (`use engine::X`), module chỉ re-export trong `mod.rs`
- **Desktop**: Feature-based structure, `mod.rs` chỉ re-export
- **Edition**: Rust 2024 cho tất cả crates

## Tài liệu

- [desktop/.rules](desktop/.rules) — Desktop structure rules (IMPORTANT!)
- [docs/gpui/](gpui/) — GPUI framework documentation