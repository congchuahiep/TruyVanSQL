use gpui::*;
use gpui_component::label::Label;
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{DataTable, TableState};
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::tab_sql_editor::session::QuerySession;
use crate::tab_sql_editor::state::OutputContent;
use crate::tab_sql_editor::table_delegate::QueryTableDelegate;

/// View hiển thị kết quả của phiên truy vấn (Query Results).
/// Sẽ được thiết kế để chỉ xuất hiện khi có dữ liệu (không phải Empty).
pub struct QueryResults {
    session: Entity<QuerySession>,
    table_state: Entity<TableState<QueryTableDelegate>>,
}

impl QueryResults {
    pub fn new(session: Entity<QuerySession>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = QueryTableDelegate::empty();
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        // Quan sát session để render lại khi output thay đổi
        cx.observe(&session, |this, _, cx| {
            let output = this.session.read(cx).output.clone();
            match &output {
                OutputContent::Query { columns, rows } => {
                    this.table_state.update(cx, |table, cx| {
                        *table.delegate_mut() = QueryTableDelegate::new(columns, rows);
                        table.refresh(cx);
                    });
                }
                _ => {
                    this.table_state.update(cx, |table, cx| {
                        *table.delegate_mut() = QueryTableDelegate::empty();
                        table.refresh(cx);
                    });
                }
            }
            cx.notify();
        })
        .detach();

        Self {
            session,
            table_state,
        }
    }
}

impl Render for QueryResults {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let output = self.session.read(cx).output.clone();

        // 1. Nếu chưa có kết quả gì, không render component này
        if matches!(output, OutputContent::Empty) {
            return div().into_any_element(); // Trả về một div trống (chiếm 0 không gian)
        }

        // 2. Xử lý hiển thị dựa trên nội dung output
        let content = match output {
            OutputContent::Empty => unreachable!(), // Đã xử lý ở trên

            OutputContent::Execution { text } => div()
                .id("results-execution")
                .w_full()
                .flex_1()
                .p_3()
                .overflow_y_scrollbar()
                .child(Label::new(text).text_sm())
                .into_any_element(),

            OutputContent::Query { .. } => div()
                .flex_1()
                .w_full()
                .min_w_0()
                .min_h_0()
                .overflow_hidden()
                .child(
                    DataTable::new(&self.table_state)
                        .stripe(true)
                        .scrollbar_visible(true, true),
                )
                .into_any_element(),

            OutputContent::Error(msg) => div()
                .id("results-error")
                .w_full()
                .flex_1()
                .p_3()
                .bg(gpui::rgba(0xff000033)) // Highlight nền đỏ nhạt cho lỗi
                .overflow_y_scrollbar()
                .child(
                    Label::new(format!("Lỗi: {}", msg))
                        .text_sm()
                        .text_color(gpui::rgba(0xff0000ff)), // Chữ màu đỏ
                )
                .into_any_element(),
        };

        // 3. Khung chứa chính (Wrapper) với Header nhỏ
        v_flex()
            .id("query-results-panel")
            .w_full()
            .h(px(250.0)) // Chiều cao cố định (Tương lai có thể làm resizable panel)
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .w_full()
                    .px_3()
                    .py_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().muted) // Nền hơi tối cho header
                    .child(
                        Label::new("Results")
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_size_0()
                    .size_full()
                    .p_2()
                    .child(content),
            )
            .into_any_element()
    }
}
