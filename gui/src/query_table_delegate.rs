//! Adapter giữa engine::QueryResult và gpui_component::DataTable.
//!
//! # Cách hoạt động
//!
//! ```text
//! Engine format:                          QueryTableDelegate:
//!
//! Column { name: "id" }            →    Column::new("id", "id").width(60.)
//! Column { name: "name" }          →    Column::new("name", "name").width(150.)
//!
//! Row { values: [Some(Integer(1)),    →  rows[0] = ["1", "John"]
//!                Some(Text("John"))] }
//!
//! Row { values: [None,                 →  rows[1] = ["NULL", "Jane"]
//!                Some(Text("Jane"))] }
//! ```
//!
//! # Virtual Scrolling
//!
//! DataTable chỉ gọi `render_td()` cho những cell đang visible trên màn hình,
//! không phải toàn bộ data. Điều này cho phép hiển thị hàng ngàn dòng
//! mà vẫn giữ hiệu suất mượt.
//!
//! # Ví dụ sử dụng
//!
//! ```ignore
//! let delegate = QueryTableDelegate::new(&columns, &rows);
//! let table_state = cx.new(|cx| TableState::new(delegate, window, cx));
//!
//! // Trong render():
//! DataTable::new(&table_state)
//!     .stripe(true)
//!     .bordered(true)
//! ```

use gpui::prelude::*;
use gpui::*;
use gpui_component::table::{Column, TableDelegate, TableState};

/// Adapter giữa engine::QueryResult và gpui_component::DataTable.
///
/// Chuyển đổi `Vec<Column>` và `Vec<Row>` từ engine thành format
/// mà `TableDelegate` trait yêu cầu.
///
/// # Fields
///
/// - `columns`: Định nghĩa cột cho DataTable (tên, độ rộng, sortable, v.v.)
/// - `rows`: Dữ liệu dòng, mỗi dòng là `Vec<String>` tương ứng với các cột
///
/// # Lifecycle
///
/// `QueryTableDelegate` được tạo mới mỗi khi query thay đổi.
/// Entity cũ bị drop, entity mới được tạo trong `OutputPanel::render_content()`.
pub struct QueryTableDelegate {
    /// Định nghĩa cột cho DataTable.
    /// Mỗi `Column` chứa key, tên hiển thị, độ rộng, và các thuộc tính khác.
    pub columns: Vec<Column>,

    /// Dữ liệu dòng, mỗi dòng là `Vec<String>`.
    /// - `rows[row_ix][col_ix]` trả về nội dung cell dưới dạng chuỗi.
    /// - `NULL` được đại diện bằng chuỗi "NULL".
    pub rows: Vec<Vec<String>>,
}

impl QueryTableDelegate {
    /// Tạo delegate từ engine columns và rows.
    ///
    /// # Conversion rules
    ///
    /// | Engine type | DataTable type |
    /// |---|---|
    /// | `engine::Column { name, ... }` | `Column::new(&name, name).width(150.0)` |
    /// | `Some(Value::Integer(v))` | `v.to_string()` |
    /// | `Some(Value::Float(v))` | `v.to_string()` |
    /// | `Some(Value::Text(v))` | `v.clone()` |
    /// | `Some(Value::Blob(v))` | `format!("<{} bytes>", v.len())` |
    /// | `None` | `"NULL"` |
    ///
    /// # Column width
    ///
    /// Hiện tại tất cả cột có width cố định 150px.
    /// Trong tương lai, có thể tính width dựa trên dữ liệu:
    /// - Width = max(tên cột, max độ dài giá trị trong cột)
    pub fn new(engine_columns: &[engine::Column], engine_rows: &[engine::Row]) -> Self {
        let columns: Vec<Column> = engine_columns
            .iter()
            .map(|col| Column::new(&col.name, col.name.clone()).width(150.0))
            .collect();

        let rows: Vec<Vec<String>> = engine_rows
            .iter()
            .map(|row| {
                row.values
                    .iter()
                    .map(|val| match val {
                        Some(v) => v.to_string(),
                        None => "NULL".to_string(),
                    })
                    .collect()
            })
            .collect();

        Self { columns, rows }
    }

    /// Tạo delegate rỗng cho khởi tạo ban đầu.
    /// Sẽ được thay thế bằng data thật khi query xong.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}

impl TableDelegate for QueryTableDelegate {
    /// Trả về số lượng cột.
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    /// Trả về số lượng dòng.
    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    /// Trả về tham chiếu đến cột thứ `col_ix`.
    ///
    /// `col_ix` được đảm bảo trong khoảng `0..columns_count`.
    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    /// Render nội dung cell tại vị trí `[row_ix][col_ix]`.
    ///
    /// Được gọi bởi DataTable chỉ cho những cell đang visible (virtual scrolling).
    /// Trả về `String` — DataTable tự động wrap trong element hiển thị.
    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        self.rows[row_ix][col_ix].clone()
    }

    /// Trả về text của cell cho mục đích export (copy, CSV).
    /// Đây là method DataTable gọi khi cần text representation.
    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        self.rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default()
    }
}
