# Nói về ba "cấp độ" Component trong GPUI

---

Trước khi nói về kiến trúc, phải hiểu rõ 3 cách tạo component trong GPUI, vì không có khái niệm nào tương đương 1:1 với React.

### Cấp 1: use_state - Hook nội bộ element

```rust
fn my_counter(colors: &Colors, window: &mut Window, cx: &mut App)
-> impl IntoElement {
    let state: Entity<Counter> = window.use_state(
        cx,
        |_window, _cx| Counter { count: 0 }
    );
    // state tự persist qua các frame, gắn với vị trí gọi hàm
}
```

React tương đương: `useState` - nhưng khác ở chỗ GPUI tạo Entity thực sự (có identity, có thể `.read(cx)`, `.update(cx, ...)`).

Dùng khi: State đơn giản, scoped trong 1 element, không cần chia sẻ.

### Cấp 2: RenderOnce - Component thuần túy presentation

```rust
#[derive(IntoElement)]
struct Counter {
    count: i32,
    on_increment: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl RenderOnce for Counter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // self bị tiêu thụ — không persist
    }
}
```

React tương đương:

```tsx
function Counter({
    count,
    onIncrement,
}: {
    count: number;
    onIncrement?: () => void;
}) {
    return <div>...</div>;
}
```

Đặc điểm: Không có state. Nhận data qua "props" (struct fields). Callbacks là `Option<Box<dyn Fn>>`. Tiêu thụ khi render - render(self) lấy ownership.

Dùng khi: Component chỉ hiển thị, không quản lý state nội bộ.

### Cấp 3: Render (Entity-backed) - Component có state và lifecycle

```rust
struct Counter { count: i32 }

impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // self persist qua các frame
        // cx.listener() cho event handlers
        // cx.spawn() cho async tasks
        // cx.subscribe() cho observing entities
    }
}
```

React tương đương: Class component với this.state - nhưng mạnh hơn nhiều vì có Entity identity.

Đặc điểm: Có internal state persist. Có `Context<Self>` cho lifecycle. Có thể observe, subscribe, spawn.

Dùng khi: Component cần internal state, async tasks, hoặc observe entities khác.

# Bản chất của Entity

Vốn GPUI gọi là Entity vì chúng không hẳn là Component, và sự thật thì chúng khác xa. Đối với React một component bao gồm:

> React Component = UI + State + Logic _(all-in-one)_

Nhưng như bạn đã đọc [overview.md](overview.md) từ trước, Entity chỉ là nơi chứa đựng State và Logic _(không có UI)_. Để một Entity có thể hiển thị UI ta cần triển khai trait Render cho nó và biến Entity thành View

Điều này ta có thể tóm gọn như sau:

```
Entity = State + Logic (KHÔNG có UI)
Render (View) = Entity + UI (Entity CÓ thể render)
RenderOnce = UI thuần túy (KHÔNG có Entity)
```

## SOS

# Ba cấp độ Component trong GPUI

---

Như đã giới thiệu trong [giới thiệu GPUI](../gpui.md), Entity là đơn vị dữ liệu do GPUI quản lý, và View là Entity có thể hiển thị (khi triển khai trait `Render`). Nhưng khi bắt tay vào xây dựng giao diện, câu hỏi đầu tiên là: **tạo một thành phần UI bằng cách nào?**

GPUI cung cấp **ba cách** khác nhau, mỗi cách phục vụ một mục đích riêng. Hiểu rõ ba cách này là chìa khóa để thiết kế kiến trúc đúng.

## Cấp 1: `use_state` - State cục bộ gắn với element

```rust
fn use_state_counter(colors: &Colors, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state: Entity<Counter> = window.use_state(
        cx,
        |_window, _cx| Counter { count: 0 }
    );

    let count = state.read(cx).count;

    div()
        .child(format!("Count: {count}"))
        .child(
            div()
                .id("increment")
                .on_click(move |_, _, cx| {
                    state.update(cx, |s, cx| {
                        s.count += 1;
                        cx.notify();
                    });
                })
        )
}
```

### Cách hoạt động

`window.use_state()` tạo ra một `Entity<T>` và **gắn nó với vị trí gọi hàm** (call site). Nghĩa là:

- Mỗi lần `use_state_counter()` được gọi trong `render()`, GPUI sẽ **tìm lại** Entity cũ dựa trên vị trí trong code, chứ không tạo mới
- Entity này persist qua các lần render - giá trị `count` được giữ nguyên
- Nếu element bị gỡ khỏi cây UI, Entity cũng bị hủy theo

### So sánh với React

```tsx
// React: useState — tương đương use_state
function Counter() {
    const [count, setCount] = useState(0);
    return <div onClick={() => setCount((c) => c + 1)}>Count: {count}</div>;
}
```

Rất giống `useState` trong React: state cục bộ, gắn với component, tự động persist.

### Khi nào dùng?

- State đơn giản, chỉ dùng trong **một chỗ duy nhất**
- Không cần chia sẻ state với component khác
- Không cần observe/subscribe Entity khác
- Quick prototype, throwaway UI

### Hạn chế

- State gắn với **vị trí gọi hàm** — nếu bạn di chuyển đoạn code, state sẽ bị reset
- Không thể truyền cho component khác (vì nó được tạo ngay trong hàm)
- Khó test riêng biệt

---

## Cấp 2: `RenderOnce` — Component thuần presentation, không có state

```rust
#[derive(IntoElement)]
struct Toolbar {
    db_path: String,
    on_new: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_open: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl Toolbar {
    /// Constructor - giống props trong React
    pub fn new(db_path: String) -> Self {
        Self {
            db_path,
            on_new: None,
            on_open: None,
        }
    }

    /// Builder method — giống truyền prop onNew={...} trong React
    pub fn on_new(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new = Some(Box::new(callback));
        self
    }

    pub fn on_open(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Box::new(callback));
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex()
            .child(Button::new("btn-new").label("New")
                .when_some(self.on_new, |el, cb| el.on_click(cb)))
            .child(Button::new("btn-open").label("Open")
                .when_some(self.on_open, |el, cb| el.on_click(cb)))
            .child(Label::new(self.db_path))
    }
}
```

### Cách hoạt động

`RenderOnce` có chữ "Once" vì **struct bị tiêu thụ khi render** — `fn render(self, ...)` lấy ownership (`self`), không phải `&mut self`. Điều này có nghĩa:

- **Mỗi frame**, parent tạo ra một instance mới của struct và truyền "props" mới
- Struct **không persist** - không có state nội bộ nào sống qua các frame
- Tất cả data đến từ struct fields (giống props trong React)
- Callbacks là `Option<Box<dyn Fn>>` - optional, truyền qua builder pattern

### So sánh với React

```tsx
// React: Functional component với props — tương đương RenderOnce
interface ToolbarProps {
    dbPath: string;
    onNew?: () => void;
    onOpen?: () => void;
}

function Toolbar({ dbPath, onNew, onOpen }: ToolbarProps) {
    return (
        <div>
            <button onClick={onNew}>New</button>
            <button onClick={onOpen}>Open</button>
            <span>{dbPath}</span>
        </div>
    );
}
```

Bản chất giống hệt: nhận props → render → hết. Không state, không side effects.

### Khi nào dùng?

- Component **chỉ hiển thị** (presentational), không quản lý state nội bộ
- Component nhận data từ parent và gọi callback ngược lên parent
- **Đa số** các UI component nhỏ (buttons, labels, panels, toolbars...)

### Hạn chế

- Không có `Context<Self>` — không thể gọi `cx.spawn()`, `cx.observe()`, `cx.subscribe()`
- Không persist state — mỗi lần render là instance mới
- Callbacks phải là `Box<dyn Fn>` — không thể dùng `cx.listener()`

### Cách truyền callback cho RenderOnce

Vì RenderOnce không có `Context<Self>`, không thể dùng `cx.listener()`. Có hai cách truyền callback:

**Cách 1: Truyền closure trực tiếp** — khi logic đơn giản

```rust
// Trong AppView::render():
let count = self.count;
.child(
    Button::new("btn")
        .on_click(move |_, _, cx| {
            // Logic đơn giản trực tiếp trong closure
            println!("Clicked! count={count}");
        })
)
```

**Cách 2: Truyền `Entity` reference** - khi cần gọi method trên Domain Service

```rust
// Trong AppView::render():
let connection = self.connection.clone(); // Entity<ConnectionService>

.child(
    Button::new("btn-open")
        .on_click(move |_, _, cx| {
            connection.update(cx, |service, cx| {
                service.open_file(cx);  // Gọi method trên Entity
            });
        })
)
```

**Cách 3: Truyền WeakEntity + callback** - khi cần gọi method trên parent View

```rust
// Trong AppView::render():
let handle = cx.entity().downgrade(); // WeakEntity<AppView>

.child(
    Toolbar::new(db_path.clone())
        .on_open(move |_, cx| {
            handle.update(cx, |app, cx| {
                app.on_open_file(cx);  // Gọi method trên AppView
            });
        })
)
```

> **Lưu ý quan trọng:** Cách 1 và 2 dùng trong `on_click` (closure tự do), Cách 3 dùng builder pattern cho RenderOnce. Cả ba đều hợp lệ, nhưng Cách 2 (truyền Entity reference) là pattern mạnh nhất và được khuyến nghị cho kiến trúc lớn.

---

## Cấp 3: `Render` — Entity-backed View có state và lifecycle

```rust
struct Sidebar {
    expanded: bool,
    selected_table: Option<String>,
    tables: Vec<TableBrief>,
}

impl Sidebar {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            expanded: true,
            selected_table: None,
            tables: Vec::new(),
        }
    }

    fn toggle(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.expanded = !self.expanded;
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .child(
                div()
                    .on_click(cx.listener(Self::toggle))  // ← cx.listener() chỉ có trong Render
                    .child(if self.expanded { "▼" } else { "▶" })
            )
            .children(self.tables.iter().map(|t| {
                div().child(&t.name)
            }))
    }
}
```

### Cách hoạt động

`Render` là cấp độ cao nhất. Struct triển khai `impl Render` trở thành **View** - một Entity có thể hiển thị. Điểm khác biệt cốt lõi:

- `fn render(&mut self, ...)` — nhận `&mut self`, có thể đọc/ghi state trực tiếp
- `Context<Self>` - cung cấp full lifecycle: `cx.listener()`, `cx.spawn()`, `cx.observe()`, `cx.subscribe()`
- Struct persist qua các frame - GPUI giữ ownership và gọi `render()` mỗi khi cần
- Có identity — `Entity<Sidebar>` có thể truyền đi, observe, subscribe

### So sánh với React

```tsx
// React: Class component — tương đương Render (Entity-backed)
class Sidebar extends React.Component {
    state = { expanded: true, selectedTable: null };

    toggle = () => {
        this.setState({ expanded: !this.state.expanded });
    };

    render() {
        return <div onClick={this.toggle}>...</div>;
    }
}
```

Nhưng GPUI mạnh hơn vì:

- Entity có identity ổn định — có thể observe từ nơi khác
- Có thể subscribe event từ Entity khác
- Có thể spawn async tasks với `cx.spawn()`
- Có thể `.read(cx)` và `.update(cx, ...)` từ bất kỳ đâu

### Khi nào dùng?

- Component cần **internal state** persist qua các frame
- Component cần **async tasks** (`cx.spawn()`)
- Component cần **observe/subscribe** Entity khác
- Component cần **identity** — được reference từ nhiều nơi

### Hạn chế

- Nhiều boilerplate hơn RenderOnce
- Cần quản lý lifecycle (observe, subscribe, detach)
- Overhead bộ nhớ cao hơn (Entity allocation)

---

## Bảng so sánh tổng hợp

| Tiêu chí                             | `use_state`        | `RenderOnce`        | `Render` (Entity-backed) |
| ------------------------------------ | ------------------ | ------------------- | ------------------------ |
| **Có state nội bộ?**                 | ✅ (gắn call site) | ❌ Không            | ✅ Có                    |
| **Persist qua frame?**               | ✅ Có              | ❌ Không (tiêu thụ) | ✅ Có                    |
| **Có `Context<Self>`?**              | ❌ Không           | ❌ Không            | ✅ Có                    |
| **`cx.spawn()` async?**              | ❌ Không           | ❌ Không            | ✅ Có                    |
| **`cx.observe()`/`cx.subscribe()`?** | ❌ Không           | ❌ Không            | ✅ Có                    |
| **`cx.listener()` cho events?**      | ❌ Không           | ❌ Không            | ✅ Có                    |
| **Có identity (Entity)?**            | ✅ (ẩn)            | ❌ Không            | ✅ Có                    |
| **Truyền đi được?**                  | ❌ Khó             | ✅ (struct)         | ✅ (Entity)              |
| **Boilerplate**                      | Thấp               | Trung bình          | Cao                      |
| **Overhead**                         | Thấp               | Thấp nhất           | Cao nhất                 |

---

## Bản chất của Entity — Tại sao không giống React Component?

Trong React, một component bao gồm tất cả:

> **React Component = UI + State + Logic** _(all-in-one)_

Như đã nói trong [giới thiệu GPUI](../gpui.md), Entity chỉ là nơi chứa State và Logic — không có UI. Để Entity hiển thị UI, cần triển khai trait `Render`. Điều này tạo ra sự phân tách rõ ràng:

```
Entity         = State + Logic (KHÔNG có UI)
Render (View)  = Entity + UI   (Entity CÓ THỂ render)
RenderOnce     = UI thuần túy  (KHÔNG có Entity)
```

---

## Domain Service Entity — Entity không render

Đây là pattern quan trọng nhất mà Zed sử dụng và cũng là pattern chúng ta sẽ áp dụng.

### Vấn đề

Khi ứng dụng lớn lên, đặt tất cả logic vào View (AppView) tạo ra **god object**:

```rust
// ❌ SAI: AppView gánh mọi thứ
pub struct AppView {
    db_url: String,
    db_path: String,
    sql_input: Entity<InputState>,
    output: OutputContent,
    is_executing: bool,
    // Tháng sau thêm: tabs, history, settings, connections...
    // AppView phình to không kiểm soát!
}

impl AppView {
    fn on_open_file(...) { /* logic mở file */ }
    fn on_execute(...) { /* logic chạy query */ }
    fn format_output(...) { /* logic format */ }
    // 50 methods nữa...
}
```

### Giải pháp: Tách logic ra Domain Service Entity

**Domain Service Entity** là Entity **không triển khai `Render`** - nó chỉ chứa state và logic, không có UI:

```rust
// ✅ ĐÚNG: Logic nằm trong Domain Service
pub struct ConnectionService {
    pub db_url: String,
    pub db_path: String,
}

impl ConnectionService {
    pub fn open_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Logic mở file nằm Ở ĐÂY, không phải AppView
        let this = cx.entity();
        cx.spawn(async move |_, cx| {
            let dialog = rfd::AsyncFileDialog::new()...;
            if let Some(file) = dialog.pick_file().await {
                this.update(cx, |service, cx| {
                    service.db_url = format!("sqlite:{}", file.path().display());
                    service.db_path = file.path().to_string_lossy().to_string();
                    cx.notify();  // ← Thông báo observers
                });
            }
        }).detach();
    }
}
// KHÔNG CÓ impl Render — đây không phải View!
```

```rust
pub struct QueryService {
    pub sql_input: Entity<InputState>,
    pub output: OutputContent,
    pub is_executing: bool,
    connection: Entity<ConnectionService>,
}

impl QueryService {
    pub fn execute(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Logic chạy query nằm Ở ĐÂY
        let sql = self.sql_input.read(cx).value().to_string();
        let db_url = self.connection.read(cx).db_url.clone();
        // ...
    }
}
// KHÔNG CÓ impl Render!
```

### AppView trở thành coordinator mỏng

```rust
pub struct AppView {
    connection: Entity<ConnectionService>,  // ← Domain Service
    query: Entity<QueryService>,              // ← Domain Service
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let connection = cx.new(|cx| ConnectionService::new(cx));
        let query = cx.new(|cx| QueryService::new(window, connection.clone(), cx));

        // Observe services → re-render khi state thay đổi
        cx.observe(&connection, |_, _, cx| cx.notify())?.detach();
        cx.observe(&query, |_, _, cx| cx.notify())?.detach();

        Self { connection, query }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let conn = self.connection.read(cx);
        let query_state = self.query.read(cx);

        v_flex()
            .size_full()
            // Truyền Entity refs cho children
            .child(Toolbar::new(conn.db_path.clone(), self.connection.clone()))
            .child(SqlEditor::new(query_state.sql_input.clone()))
            .child(ExecuteBar::new(query_state.is_executing, self.query.clone()))
            .child(OutputPanel::new(&query_state.output))
    }
}
```

### Sự phân tách trách nhiệm

```
┌──────────────────────────────────────────────────────────────┐
│                      AppView (Coordinator)                    │
│  Chỉ tạo Entity, observe, và truyền Entity refs cho children │
│  KHÔNG chứa business logic                                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐         │
│  │ ConnectionService    │    │ QueryService          │         │
│  │ (Entity, KHÔNG render)│    │ (Entity, KHÔNG render)│         │
│  │                     │    │                     │         │
│  │ State: db_url,      │    │ State: output,      │         │
│  │        db_path      │    │        is_executing │         │
│  │                     │    │                     │         │
│  │ Logic: open_file(), │    │ Logic: execute(),    │         │
│  │        new_db(),    │    │        run_query()  │         │
│  │        in_memory()  │    │                     │         │
│  └─────────────────────┘    └─────────────────────┘         │
│           ▲                          ▲                      │
│           │ Entity reference          │ Entity reference     │
│           │                          │                      │
│  ┌────────┴──────────┐    ┌─────────┴──────────┐           │
│  │ Toolbar            │    │ ExecuteBar          │           │
│  │ (RenderOnce)       │    │ (RenderOnce)        │           │
│  │                    │    │                     │           │
│  │ Hiển thị db_path   │    │ Hiển thị status     │           │
│  │ Gọi:               │    │ Gọi:                │           │
│  │  connection.update( │    │  query.update(cx,   │           │
│  │   cx, |s| s.open_  │    │   |q| q.execute()) │           │
│  │   file())          │    │                     │           │
│  └────────────────────┘    └─────────────────────┘           │
└──────────────────────────────────────────────────────────────┘
```

### Quy tắc phân bổ trách nhiệm

| Trách nhiệm                             | Nằm ở đâu                 | Ví dụ                                            |
| --------------------------------------- | ------------------------- | ------------------------------------------------ |
| **State + Logic** (mở file, chạy query) | Domain Service Entity     | `ConnectionService`, `QueryService`              |
| **Tạo & wire Entity**                   | Root View (AppView)       | `cx.new()`, `cx.observe()`                       |
| **Hiển thị UI, gọi method trên Entity** | RenderOnce / Render       | `Toolbar`, `ExecuteBar`                          |
| **Re-render khi state thay đổi**        | Root View observe Service | `cx.observe(&service, \|_, _, cx\| cx.notify())` |

---

## Sơ đồ quyết định: Khi nào dùng cái gì?

```
Component của bạn có cần state nội bộ không?
├── KHÔNG
│   └──→ Dùng RenderOnce
│       (Toolbar, Label, OutputPanel, ExecuteBar...)
│
└── CÓ
    ├── State đó có cần hiển thị UI không?
    │   ├── CÓ
    │   │   └──→ Dùng Render (Entity-backed View)
    │   │       (Sidebar, DataGrid, TabManager...)
    │   │
    │   └── KHÔNG
    │       └──→ Dùng Entity thuần (Domain Service)
    │           (ConnectionService, QueryService, HistoryStore...)
    │
    └── State chỉ dùng cục bộ trong 1 element?
        └──→ Dùng use_state
            (counter đơn giản, toggle cục bộ...)
```

### Ví dụ phân loại trong dự án SQL Client

| Thành phần             | Loại                        | Lý do                                               |
| ---------------------- | --------------------------- | --------------------------------------------------- |
| `ConnectionService`    | Entity thuần (không Render) | State + Logic, không cần UI                         |
| `QueryService`         | Entity thuần (không Render) | State + Logic, không cần UI                         |
| `Toolbar`              | RenderOnce                  | Chỉ hiển thị, gọi method trên Entity                |
| `SqlEditor`            | RenderOnce                  | Bọc `InputState` (Entity có sẵn), không state riêng |
| `ExecuteBar`           | RenderOnce                  | Chỉ hiển thị nút + status, gọi method trên Entity   |
| `OutputPanel`          | RenderOnce                  | Chỉ hiển thị text, không state riêng                |
| `AppView`              | Render (Entity-backed View) | Tạo & wire Entity, observe, render layout           |
| `Sidebar` (tương lai)  | Render (Entity-backed View) | Cần internal state: expanded, selected_table        |
| `DataGrid` (tương lai) | Render (Entity-backed View) | Cần internal state: scroll position, selection      |

---

## Giao tiếp giữa các thành phần

### Pattern 1: Entity reference (khuyến nghị)

RenderOnce component nhận `Entity<T>` qua constructor, gọi `.read(cx)` và `.update(cx, ...)` trực tiếp:

```rust
// Parent truyền Entity reference
.child(Toolbar::new(self.connection.clone()))

// RenderOnce component sử dụng Entity
impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let db_path = self.connection.read(cx).db_path.clone();  // Đọc state

        h_flex()
            .child(Button::new("btn-open")
                .on_click(move |_, _, cx| {
                    self.connection.update(cx, |s, cx| s.open_file(cx));  // Gọi method
                }))
    }
}
```

**Ưu điểm:** Type-safe, trực tiếp, không cần event system.

### Pattern 2: Callback (cho logic đơn giản)

RenderOnce component nhận callback qua builder pattern:

```rust
// Parent truyền callback
let handle = cx.entity().downgrade(); // WeakEntity<AppView>
.child(
    Toolbar::new(db_path.clone())
        .on_open(move |_, cx| {
            handle.update(cx, |app, cx| app.on_open_file(cx));
        })
)

// RenderOnce component sử dụng callback
impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex()
            .child(Button::new("btn-open")
                .when_some(self.on_open, |el, cb| el.on_click(cb)))
    }
}
```

**Ưu điểm:** Component không cần biết về Entity cụ thể, dễ tái sử dụng.
**Nhược điểm:** Nhiều callback = nhiều `Box<dyn Fn>` = nhiều allocation.

### Pattern 3: EventEmitter + Subscribe (cho communication phức tạp)

Khi cần communication hai chiều hoặc nhiều subscriber:

```rust
// 1. Định nghĩa event enum
pub enum SidebarEvent {
    TableSelected(String),
    TableExpanded(bool),
}

// 2. Entity triển khai EventEmitter
impl EventEmitter<SidebarEvent> for Sidebar {}

// 3. Parent subscribe
cx.subscribe(&sidebar, |app, _sidebar, event, cx| {
    match event {
        SidebarEvent::TableSelected(table) => {
            // Xử lý event
        }
        _ => {}
    }
}).detach();
```

**Ưu điểm:** Decoupled, nhiều subscriber, hai chiều.
**Nhược điểm:** Nhiều boilerplate, khó debug.

### Khi nào dùng pattern nào?

```
Logic đơn giản, 1-2 callbacks → Pattern 2 (Callback)
Logic phức tạp, cần gọi method trên Entity → Pattern 1 (Entity reference)
Communication hai chiều, nhiều subscriber → Pattern 3 (EventEmitter)
```

---

## Tóm tắt

```
Entity         = State + Logic (KHÔNG có UI)     → Giống React Context/Provider
Render (View)  = Entity + UI   (CÓ THỂ render)   → Giống React Class Component
RenderOnce     = UI thuần túy  (KHÔNG có Entity)  → Giống React Functional Component (props-only)
use_state      = State cục bộ  (gắn call site)     → Giống React useState
```

Nguyên tắc thiết kế:

1. **Logic thuộc về Domain Service Entity** — không thuộc View
2. **View chỉ hiển thị và gọi method trên Entity** — không chứa business logic
3. **RenderOnce cho component không state** — đa số UI nhỏ
4. **Render cho component có state** — chỉ khi thực sự cần lifecycle
5. **Entity thuần cho Domain Service** — state + logic, không UI
6. **AppView là coordinator mỏng** — tạo, wire, observe, layout

```

---

Đây là nội dung đề xuất cho `component.md`. Bạn có muốn tôi điều chỉnh phần nào trước khi bạn chép vào file?
```
