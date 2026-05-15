# Giới thiệu

---

GPUI là một thư viện UI được viết bằng Rust bởi đội ngũ Zed Editor. Điểm nổi trội của GPUI nó là một thư viện _GPU-accelerated_ và _Hybrid immediate + retained mode_

Để hiểu định nghĩa này, chúng ta cần phân tích từng thuật ngữ:

### GPU-accelerated

Khác với các framework UI truyền thống _(như GTK, Qt)_ render bằng CPU, GPUI sử dụng GPU để vẽ toàn bộ giao diện. Nhờ đó mà UI thực sự mượt mà, phải gọi là có thể chạy 60fps!

> So sánh với React: React DOM render bằng CPU _(tính toán DOM diff, apply styles, paint)_. GPUI đẩy toàn bộ việc "vẽ" sang GPU, tương tự như cách WebGL/Canvas hoạt động trong browser

### Hybrid immediate + retained mode

Trong lập trình đồ họa và giao diện _(GUI)_, Retained Mode và Immediate Mode là hai phương thức quản lý dữ liệu và vẽ hình ảnh lên màn hình hoàn toàn khác nhau

Điểm khác biệt cốt lõi nằm ở ai là người giữ trạng thái (state) của giao diện: thư viện đồ họa hay mã nguồn ứng dụng của bạn

#### a. Retained Mode - Chế độ lưu giữ

Trong Retained Mode, bạn định nghĩa các đối tượng _(nút bấm, hình vuông, văn bản)_ và gửi chúng cho thư viện đồ họa. Thư viện này sẽ lưu giữ một bản sao _(scene graph, hoặc trong web thường thấy là DOM)_ của toàn bộ giao diện trong bộ nhớ

- **Cách hoạt động:** Bạn tạo một đối tượng _(ví dụ: `new Button()`)_, thiết lập các thuộc tính và thêm nó vào "cây" giao diện. Thư viện sẽ tự động vẽ lại đối tượng này mỗi khung hình

- **Cập nhật state:** Khi muốn đổi màu nút, bạn chỉ cần thay đổi thuộc tính của đối tượng đó. Thư viện sẽ **tự biết** phần nào cần vẽ lại

> _**Ví dụ.** HTML/DOM (Web), WPF (Windows), SwiftUI, Jetpack Compose, Qt_

**Ưu điểm:**

- Thư viện tự xử lý các tác vụ phức tạp như tối ưu hóa việc vẽ _(chỉ vẽ lại vùng thay đổi)_
- Phù hợp với các giao diện tĩnh hoặc có cấu trúc phân cấp phức tạp

**Nhược điểm:**

- Tốn bộ nhớ để duy trì toàn bộ mô hình giao diện
- Khó đồng bộ hóa trạng thái giữa logic ứng dụng và giao diện _(dễ xảy ra lỗi "out of sync")_

#### b. Immediate Mode - Chế độ tức thời

Trong Immediate Mode, thư viện đồ họa không lưu giữ bất kỳ thông tin nào về các đối tượng. Mỗi khung hình, ứng dụng của bạn phải ra lệnh vẽ lại toàn bộ mọi thứ từ đầu

- **Cách hoạt động:** Bạn gọi một hàm vẽ ngay trong vòng lặp chính của ứng dụng _(ví dụ: `if (DrawButton("Click Me")) { ... }`)_. Nếu bạn ngừng gọi hàm này ở khung hình tiếp theo, nút bấm sẽ biến mất ngay lập tức

- **Cập nhật:** Bạn không _"thay đổi thuộc tính"_ của nút; bạn chỉ đơn giản là truyền dữ liệu mới vào hàm vẽ ở khung hình tiếp theo

> _**Ví dụ.** Dear ImGui, OpenGL (phần glBegin/glEnd cũ), Unity OnGUI_

**Ưu điểm:**

- Cực kỳ đơn giản để viết mã: giao diện luôn phản ánh đúng trạng thái hiện tại của dữ liệu
- Lý tưởng cho các công cụ debug, trình biên tập trong game _(game editors)_ hoặc giao diện thay đổi liên tục
  **Nhược điểm:**
- Tốn CPU vì phải xử lý logic và dựng lệnh vẽ cho toàn bộ UI ở mỗi khung hình _(thường là 60 lần/giây)_
- Khó thiết kế các giao diện có bố cục _(layout)_ tự động co giãn phức tạp

#### Sự kết hợp giữa hai phương thức quản lý GUI

# Các thành phần căn bản trong GPUI

---

### Entity

Entity là một đơn vị dữ liệu mà GPUI quản lý. Mỗi Entity lưu trữ trạng thái _(state)_ riêng _(state này thường là một struct)_ và GPUI sẽ tự động theo dõi sự thay đổi của nó để cập nhật giao diện. Nhờ GPUI nắm quyền sở hữu Entity, bạn có thể:

- Dùng chung một Entity ở nhiều nơi trong ứng dụng
- GPUI tự động phát hiện thay đổi và render lại giao diện tương ứng
- GPUI tự quản lý vòng đời _(tạo, cập nhật, hủy)_ của Entity

Vì GPUI sở hữu Entity, bạn chỉ có thể mutate dữ liệu bằng cách phương thức có sẵn mà Entity cung cấp, một trong hai phương thức căn bản là:

- `.read()`: đọc dữ liệu
- `.update()`: cập nhật dữ liệu, thường sẽ sử dụng `.notify()` để ép rerender lại
- `.notify()`: rerender lại giao diện

```rust

// Bước 0: Tạo mộ struct để định nghĩa dữ liệu state
struct Counter {
    count: i32,
}

// ... Đâu đó trong Application context

// Bước 1: Tạo Entity
let counter: Entity<Counter> = cx.new(|cx| Counter { count: 0 });

// Bước 2: Đọc dữ liệu
let data = counter.read(cx);
println!("Count: {}", data.count);  // In ra: Count: 0

// Bước 3: Ghi dữ liệu
counter.update(cx, |counter, cx| {
    counter.count += 1;
    cx.notify(); // Báo cho GPUI: "Dữ liệu đã thay đổi, hãy bắt đầu rerender"
});
```

### View - Entity có thể hiển thị

View là Entity có thể hiển thị lên màn hình. Việc tách biệt giữa View và Entity là vì không phải Entity lúc nào cũng cần được hiển thị, ví dụ:

- `Entity<DatabaseConnection>` - kết nối database, không cần vẽ
- `Entity<UserSettings>` - cài đặt người dùng, không cần vẽ
- `Entity<Counter>` - bộ đếm, cần vẽ → đây là View

Để biến một Entity trở thành một View, bạn đơn giản là triển khai trait `Render` cho struct của Entity đó

```rust
struct Counter {
    count: i32,
}

// Implement Render trait → Counter trở thành View
impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Hàm này trả về "cây giao diện"
        // Mỗi khi state thay đổi, hàm này được gọi lại
        div()
            .child(format!("Count: {}", self.count))
    }
}
```

Trait Render yêu cầu triển khai hàm `render` để biết được cách thức Entity đó muốn vẽ trên màn hình

View sẽ được gọi rerender trong 4 trường hợp sau:

1. Khởi tạo: Khi view được đặt vào trong windows
2. Khi gọi `cx.notify()` thủ công: thường xảy ra khi bạn cập nhật state và thủ công gọi hàm này để thông báo rằng cần rerender lại view
3. Khi cửa sổ thay đổi kích thước
4. Khi theme thay đổi
