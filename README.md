# TruyVanSQL

TruyVanSQL là một ứng dụng quản lý và truy vấn SQL hiện đại, mạnh mẽ, được xây dựng hoàn toàn bằng ngôn ngữ lập trình Rust. Ứng dụng tập trung vào hiệu năng cao, trải nghiệm người dùng mượt mà với giao diện đồ họa (GUI) dựa trên framework GPUI.

## Kiến trúc dự án

Dự án được tổ chức theo mô hình Rust Workspace với các thành phần chính:

- **`engine/`**: Thư viện lõi xử lý logic kết nối Database, thực thi truy vấn và quản lý Schema. Sử dụng `sqlx` làm driver chính. Hiện tại hỗ trợ SQLite và đang trong lộ trình hỗ trợ PostgreSQL/MySQL.
- **`desktop/`**: Ứng dụng GUI chính sử dụng framework **GPUI** (được phát triển bởi đội ngũ tạo ra Zed editor). Đây là nơi chứa toàn bộ logic về giao diện, quản lý tab và Data Grid.
- **`cli/`**: Giao diện dòng lệnh (đang phát triển).
- **`docs/`**: Chứa tài liệu kỹ thuật, thiết kế và kế hoạch phát triển (`plan.md`).

## Công nghệ sử dụng

- **Language**: Rust (Edition 2024).
- **GUI Framework**: [GPUI](https://github.com/zed-industries/zed).
- **Database Engine**: [SQLx](https://github.com/launchbadge/sqlx).
- **Runtime**: [Tokio](https://tokio.rs/).
- **Component Library**: [gpui-component](https://github.com/longbridge/gpui-component).

## Hướng dẫn phát triển

### 1. Cài đặt môi trường

Đảm bảo bạn đã cài đặt Rust toolchain (phiên bản mới nhất hỗ trợ Edition 2024).

### 2. Xây dựng dự án

```bash
cargo build
```

### 3. Chạy ứng dụng

```bash
cargo run -p desktop
```

### 4. Kiểm thử

Dự án có hệ thống test tích hợp mạnh mẽ trong `engine/tests`:

```bash
cargo test
```

## Quy ước phát triển

- **Edition**: Luôn sử dụng Rust edition 2024.
- **Dependency**: Các dependency dùng chung nên được khai báo tại `[workspace.dependencies]` trong file `Cargo.toml` ở thư mục gốc.
- **UI Design**: Giao diện được thiết kế theo dạng Tab-based. Mọi tương tác nặng với Database phải được thực hiện bất đồng bộ (`async`) để tránh gây treo UI.
- **Data Grid**: Sử dụng Virtual Scrolling để tối ưu hóa việc hiển thị lượng dữ liệu lớn (hàng trăm ngàn dòng).
