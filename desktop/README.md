# Desktop — Ứng dụng GUI

Chứa giao diện người dùng (GUI) viết bằng GPUI framework.

## Modules

```
desktop/src/
├── main.rs, action.rs       # Entry points
├── component/               # Custom components (Tab, SidebarMenuItem)
├── connection/              # DatabaseConnection + ConnectionStore
├── panel/                   # All UI panels
│   ├── sidebar.rs          # Explorer (browse tables)
│   ├── titlebar.rs        # Custom titlebar
│   ├── tab/               # Tab infrastructure
│   │   ├── tab_bar.rs    # Render tabs
│   │   ├── tab_item.rs   # TabItem trait
│   │   └── tab_manager.rs # Manage tabs
│   └── tab_content/       # Tab implementations
│       ├── sql_editor/   # SQL Editor tab
│       └── table_viewer/ # Table Viewer tab
├── shared/                  # SmartDataGrid component
├── theme/                   # Theme system + Mica
├── window/                  # Connection dialog
└── workspace/               # Root view
```

## Module tác dụng

| Module        | Mô tả                                                                 |
| ------------- | --------------------------------------------------------------------- |
| `connection/` | Quản lý kết nối database (DatabaseConnection entity, ConnectionStore) |
| `panel/`      | UI panels: Sidebar, Titlebar, Tab system, Tab content                 |
| `workspace/`  | Root view điều phối các panel                                         |
| `shared/`     | SmartDataGrid cho table viewer/editing                                |
| `window/`     | Connection dialog (secondary window)                                  |
| `theme/`      | Theme system với hot-reload, Mica backdrop                            |

## Thiết kế quan trọng

### Polymorphic Tabs

"Everything is a Tab Item" — SQL Editor và Table Viewer đều implement `TabItem` trait, quản lý bởi `TabManager`.

### Feature-based structure

Mỗi module là một feature domain độc lập. Xem [desktop/.rules](desktop/.rules) cho chi tiết.

## Rules

**IMPORTANT:** Đọc [desktop/.rules](desktop/.rules) trước khi sửa đổi desktop crate.

Key rules:

- `mod.rs` chỉ re-export, KHÔNG định nghĩa logic
- Single file module không cần `mod.rs`
- File name = snake_case của struct chính
- Độ sâu tối đa 4 cấp từ `src/`
