# Hệ thống Action trong GPUI

---

GPUI Action là một trong những cơ chế mạnh mẽ và đặc trưng nhất của framework này. Nó đóng vai trò là "xương sống" để kết nối giữa ý định của người dùng (User Intent) và mã thực thi thực tế (Implementation), giúp tách biệt hoàn toàn giao diện (UI) khỏi logic nghiệp vụ.

## 1. Action là gì?

Nói một cách đơn giản, **Action là một mệnh lệnh (Command)**.

Thay vì gọi trực tiếp một hàm xử lý khi người dùng nhấn phím hoặc click chuột, GPUI sẽ gửi một "gói tin" (Action) vào hệ thống. Hệ thống sau đó sẽ tự động tìm kiếm xem thành phần nào trên màn hình đang sẵn sàng xử lý gói tin đó.

### Tại sao cần Action?

- **Tính nhất quán (Consistency):** Một hành động "Lưu" có thể được kích hoạt từ menu, phím tắt `Ctrl + S`, hoặc nút bấm trên màn hình. Tất cả đều sẽ gửi cùng một Action `Save`.
- **Giải nối (Decoupling):** Nút bấm không cần biết hàm `save_to_db()` nằm ở đâu. Nó chỉ cần biết nó muốn "Save".
- **Định tuyến thông minh (Routing):** Action tự động tìm đến đúng đối tượng đang có tiêu điểm (Focus).

---

## 2. Các khái niệm cốt lõi

### A. Khai báo Action

Trong Rust, mỗi Action là một struct thực thi trait `gpui::Action`. Tuy nhiên, GPUI cung cấp macro `actions!` để tạo hàng loạt Action một cách nhanh chóng.

```rust
// Khai báo trong một module bất kỳ
gpui::actions!(my_namespace, [Calculate, Clear, Save]);
```

### B. Định tuyến (Routing)

Đây là phần "ma thuật" nhất của GPUI. Khi một Action được phát đi (dispatch):

1. GPUI bắt đầu từ **Phần tử đang có Focus** (ví dụ: một ô nhập liệu).
2. Nếu phần tử đó có đăng ký `.on_action` cho Action này, nó sẽ thực thi.
3. Nếu không, GPUI đi ngược lên **Phần tử cha**, rồi cha của cha... (Bubble up).
4. Cuối cùng, nếu không ai nhận, nó sẽ kiểm tra ở mức độ **Toàn cục (Global)**.

### C. Ngữ cảnh (Context)

Action có thể được giới hạn trong một không gian cụ thể. Ví dụ: Phím `Enter` trong Editor thì dùng để xuống dòng, nhưng phím `Enter` trong một hộp thoại xác nhận thì dùng để "Đồng ý". GPUI sử dụng `key_context` để phân biệt điều này.

---

## 3. Cách sử dụng chi tiết

### 1. Đăng ký Phím tắt (Key Bindings)

Phím tắt là cách phổ biến nhất để kích hoạt Action.

```rust
cx.bind_keys([
    KeyBinding::new("ctrl-s", Save, Some("editor-context")),
    KeyBinding::new("enter", Confirm, Some("dialog-context")),
]);
```

### 2. Lắng nghe Action trong View

Để một View có thể xử lý Action, ta sử dụng hàm `.on_action`.

```rust
impl Render for MyView {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("editor-context") // Xác định không gian cho phím tắt
            .on_action(cx.listener(Self::on_save)) // Đăng ký hàm xử lý
            .child("Nội dung editor")
    }
}

impl MyView {
    fn on_save(&mut self, action: &Save, _w: &mut Window, cx: &mut Context<Self>) {
        println!("Đã nhận lệnh Lưu!");
    }
}
```

### 3. Dispatch Action thủ công (Nút bấm & Code)

Đôi khi bạn muốn kích hoạt Action từ code (ví dụ: sau khi click vào `Button`).

#### Từ Window (Bắt đầu tìm từ tiêu điểm hiện tại)

Đây là cách phổ biến nhất cho các nút bấm trên Toolbar.

```rust
Button::new("save-btn")
    .on_click(|_, window, cx| {
        window.dispatch_action(Box::new(Save), cx);
    })
```

#### Từ AppContext (Gửi toàn cầu)

Dùng cho các lệnh không phụ thuộc vào Focus như `Quit` hay `ToggleTheme`.

```rust
cx.dispatch_action(Box::new(Quit));
```

---

## 4. Action có tham số (Parameterized Actions)

Một điểm mạnh khác là Action có thể mang theo dữ liệu (nhờ việc nó là một struct và hỗ trợ Serialization).

```rust
// Khai báo Action có tham số
#[derive(serde::Deserialize, gpui::IntoElement)]
struct OpenFile {
    path: String,
}

// Khi dispatch
window.dispatch_action(Box::new(OpenFile { path: "/etc/hosts".into() }), cx);

// Khi xử lý
fn on_open_file(&mut self, action: &OpenFile, ...) {
    println!("Mở file: {}", action.path);
}
```

---

## 5. Quy tắc và Lưu ý quan trọng

1.  **Luôn dùng `Box::new()` khi dispatch:** Hàm `dispatch_action` yêu cầu một Trait Object.
2.  **Thứ tự ưu tiên:** Action cục bộ (gần Focus nhất) luôn được thực hiện trước Action toàn cục.
3.  **Dừng truyền tin (Stop Propagation):** Nếu bạn xử lý một Action và không muốn các lớp cha nhận được nó nữa, GPUI sẽ mặc định dừng lại sau khi hàm listener kết thúc.
4.  **Đặt tên Namespace:** Nên đặt tên namespace (trong macro `actions!`) trùng với tên module hoặc tính năng để tránh xung đột (ví dụ: `grid::Save`, `editor::Save`).

Hệ thống Action chính là thứ biến các ứng dụng GPUI thành những công cụ cực kỳ linh hoạt và thân thiện với bàn phím (Keyboard-centric). Việc nắm vững nó sẽ giúp bạn xây dựng được những ứng dụng có độ phức tạp cao mà vẫn giữ được mã nguồn sạch sẽ.
