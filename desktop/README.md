# Note

- Hiện tại DataGrid chưa thể bấm enter để vào editable cell được, và sau khi commit_edit xong thì nó không focus vào bảng lại (khiến cho không thể di chuyển selected cell được)

- Thay vì input trong cell thì ta có thể mở một popup tại vị trí của cell đó và trong popup có input giá trị mới của cell sẽ hay hơn (cho phép khung edit linh hoạt hơn, hiển thị lỗi validate tốt hơn) (ta làm cách này cũng là do Table của gpui_component khó để custom style của select trong bảng)

# Desktop Crate - SQL Client Architecture

Crate `desktop` chứa giao diện người dùng (GUI) và logic quản lý trạng thái của ứng dụng SQL Client, được xây dựng bằng framework [GPUI](https://gpui.rs/).

Kiến trúc của crate này được thiết kế dựa trên các mô hình của các phần mềm chuyên nghiệp như **Zed Editor** hay **DBeaver**, tập trung vào tính mở rộng (Scalability) và sự cô lập (Isolation) thông qua mô hình **Feature-based** và **Polymorphic Tabs** (Tab Đa hình).

## 1. Triết lý Thiết kế

### A. Feature-based Structure (Cấu trúc theo Tính năng)

Thay vì gom nhóm tất cả các tệp theo loại (ví dụ: tất cả service vào thư mục `service/`, tất cả giao diện vào thư mục `panel/`), dự án được chia thành các cụm tính năng độc lập.
Mỗi thư mục tính năng (ví dụ: `tab_sql_editor`) sẽ tự định nghĩa cả logic (State/Service) lẫn giao diện (View) của riêng nó. Điều này giúp mã nguồn dễ bảo trì, và khi cần thêm một tính năng mới (như `tab_chart` hay `tab_settings`), ta chỉ cần tạo một thư mục mới mà không làm ảnh hưởng đến các phần khác.

### B. Polymorphic Tabs (Hệ thống Tab Đa hình)

Triết lý "Everything is a Tab Item".
`TabManager` ở lớp vỏ (Workspace) hoàn toàn "mù" về nội dung bên trong của một Tab. Nó quản lý một danh sách các `AnyTab` (sử dụng cơ chế Type Erasure - xóa kiểu của Rust).
Bất kỳ cấu trúc dữ liệu nào implement trait `TabItem` (định nghĩa cách lấy tiêu đề `tab_title`,...) đều có thể được ném vào `TabManager` để hiển thị. GPUI sẽ tự động biết cách gọi hàm `render()` tương ứng thông qua `AnyView`.

## 2. Cấu trúc Thư mục

```text
desktop/src/
├── main.rs                  # Điểm khởi chạy ứng dụng, thiết lập phím tắt và mở cửa sổ.
├── action.rs                # Định nghĩa các Action toàn cục (Phím tắt, Menu commands).
│
├── connection/              # [Domain] Quản lý kết nối Database
│   ├── model.rs             # DatabaseConnection: Thực thể quản lý vòng đời 1 kết nối & chứa SqlClient (Pool).
│   └── store.rs             # ConnectionStore: Quản lý danh sách (1-N) DatabaseConnection.
│
├── workspace/               # [Shell] Bộ khung giao diện và Điều phối
│   ├── mod.rs               # Workspace: Coordinator cấp cao nhất, chứa Sidebar và TabBar.
│   ├── sidebar.rs           # Explorer: Hiển thị danh sách Database dạng cây (Lazy loading tables).
│   ├── title_bar.rs         # Thanh Menu ở trên cùng.
│   ├── tab_bar.rs           # Thanh chứa các tab hiện tại.
│   ├── tab_manager.rs       # Dịch vụ quản lý danh sách các AnyTab (đóng/mở/chuyển tab).
│   └── tab_item.rs          # Định nghĩa Trait TabItem.
│
└── tab_sql_editor/          # [Feature] Cụm tính năng SQL Editor
    ├── mod.rs               # SqlEditorTab: View chính của Tab (implement TabItem). Gom Editor, Toolbar, Results.
    ├── session.rs           # QuerySession: Logic và State cục bộ của Tab (SQL input, Kết quả, Thực thi).
    ├── editor.rs            # Giao diện soạn thảo SQL (Input).
    ├── toolbar.rs           # Nút Execute và hiển thị trạng thái đang chạy.
    ├── results.rs           # Khung chứa kết quả (Lazy appearance).
    └── table_delegate.rs    # Adapter chuyển đổi dữ liệu từ Engine sang DataGrid của GPUI.
```

## 3. Luồng Dữ liệu (Data Flow)

Kiến trúc đảm bảo sự cô lập cao, mỗi Tab tự sống vòng đời của nó mà không phụ thuộc vào trạng thái "Global Active".

1. **Khởi tạo Kết nối:**
    - Người dùng nhấn "Open File" -> `Workspace` gọi `ConnectionStore::add_connection`.
    - `ConnectionStore` tạo một `DatabaseConnection` và ngay lập tức gọi lệnh `.connect()`.
    - `DatabaseConnection` dùng `cx.spawn` mở Pool và gọi Engine để fetch `tables`, sau đó lưu vào bộ nhớ cache của nó.

2. **Hiển thị Explorer:**
    - `Explorer` observe `ConnectionStore`. Khi có kết nối mới, nó vẽ ra một Node cha (Database).
    - Người dùng click mở rộng Database -> Nếu chưa có bảng (Lazy), gọi `refresh_metadata`. Vẽ ra các Node con (Tables) dựa trên cache.

3. **Mở Tab mới:**
    - Click đúp vào một Table -> `Explorer` tạo một `SqlEditorTab` (truyền bản sao tham chiếu của `DatabaseConnection` vào đó).
    - `Explorer` gọi `TabManager::open_tab`.
    - `TabManager` bọc `SqlEditorTab` thành `AnyTab` và vẽ lên `TabBar`.

4. **Thực thi Truy vấn (Trong 1 Tab):**
    - Mỗi `SqlEditorTab` có một `QuerySession` nội bộ.
    - Nhấn "Execute" -> `QuerySession` đọc SQL -> Lấy `SqlClient` (Arc) từ `DatabaseConnection` của riêng nó -> Chạy lệnh qua Engine.
    - Kết quả trả về -> Cập nhật `OutputContent` nội bộ.
    - `QueryResults` observe `QuerySession` -> Vẽ DataGrid hoặc thông báo lỗi ra màn hình.
    - (Đặc biệt) Nếu là lệnh DDL (như CREATE TABLE), Tab sẽ tự động yêu cầu `DatabaseConnection` refresh lại metadata, khiến Explorer cũng tự động cập nhật.

## 4. Nguyên tắc Lập trình (Best Practices)

- **Biến `cx` trong Closure:** Trong GPUI, tuyệt đối không _capture_ biến `cx` từ môi trường bên ngoài vào các event handler (như `on_click`). Luôn sử dụng tham số `cx` được framework truyền trực tiếp vào tham số của closure (`move |_, _, cx|`).
- **Naming Convention (Zed Style):**
    - **Entity Field:** Đặt tên trơn (ví dụ: `workspace: Entity<Workspace>`).
    - **Read Data:** Dùng tên trơn, shadow lại tên (ví dụ: `let workspace = self.workspace.read(cx);`).
    - **Clone cho Closure:** Dùng hậu tố tùy ngữ cảnh nếu bị trùng (ví dụ: `let workspace_handle = self.workspace.clone();`).
- **Không có "Global Active DB":** Các Tab không phụ thuộc vào việc bạn đang click vào Database nào trên Explorer. Database mà Tab sử dụng được cố định ngay từ lúc Tab sinh ra. Điều này cho phép mở nhiều Tab trỏ vào nhiều Database khác nhau cùng lúc.
