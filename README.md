# TruyVanSQL

SQL client đa nền tảng viết bằng Rust, sử dụng GPUI framework (Zed Editor) cho UI.

## Tính năng

- **SQL Editor** với tab management
- **Table Viewer** với editable data grid
- **Connection Manager** — SQLite & PostgreSQL
- **Custom Titlebar** với Mica backdrop (Windows)
- **Theme System** với hot-reload

## Cấu trúc dự án

```
truyvansql/
├── engine/                 # Thư viện lõi (library)
├── desktop/               # Ứng dụng GUI (binary)
├── assets/               # Icons (RustEmbed)
├── cli/                  # CLI tool (WIP)
├── docs/                 # Tài liệu kỹ thuật
└── themes/               # JSON theme files
```

## Công nghệ

| Thành phần | Công nghệ                  |
| ---------- | -------------------------- |
| UI         | GPUI, gpui-component       |
| Database   | sqlx (SQLite + PostgreSQL) |
| Async      | tokio                      |
| Icons      | RustEmbed                  |

## Phát triển

```bash
# Build
cargo build

# Run desktop
cargo run -p desktop

# Test
cargo test
```

## Tài liệu

| File                                         | Mô tả                   |
| -------------------------------------------- | ----------------------- |
| [docs/architecture.md](docs/architecture.md) | Kiến trúc hệ thống      |
| [docs/plan.md](docs/plan.md)                 | Roadmap & trạng thái    |
| [docs/gpui/](docs/gpui/)                     | GPUI framework docs     |
| [desktop/.rules](desktop/.rules)             | Desktop structure rules |
