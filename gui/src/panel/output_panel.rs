use gpui::prelude::*;
use gpui::*;
use gpui_component::label::Label;
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{DataTable, TableDelegate, TableEvent, TableState};
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::action::datagrid::Copy;
use crate::query_table_delegate::QueryTableDelegate;
use crate::service::query_service::QueryService;
use crate::state::OutputContent;

pub struct OutputPanel {
    query: Entity<QueryService>,

    table_state: Entity<TableState<QueryTableDelegate>>,
}

impl OutputPanel {
    pub fn new(query: Entity<QueryService>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = QueryTableDelegate::empty();
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        cx.observe(&query, |this, _, cx| {
            let output = this.query.read(cx).output.clone();
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

        cx.subscribe_in(&table_state, window, Self::on_table_event)
            .detach();

        Self { query, table_state }
    }

    fn on_table_event(
        &mut self,
        _table: &Entity<TableState<QueryTableDelegate>>,
        event: &TableEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::SelectCell(row_ix, col_ix) => {
                // Cell được chọn — có thể hiển thị thông tin ở status bar
                println!("Selected cell: ({}, {})", row_ix, col_ix);
            }
            TableEvent::DoubleClickedCell(row_ix, col_ix) => {
                // Double-click cell — có thể dùng cho edit trong tương lai
                println!("Double-clicked cell: ({}, {})", row_ix, col_ix);
            }
            _ => {}
        }
    }

    /// Copy cell hoặc row đang selected vào clipboard.
    /// - Nếu có cell selected → copy cell value
    /// - Nếu có row selected → copy cả row (tab-separated)
    /// - Nếu không có gì → không làm gì (propagate Ctrl+C đến InputState)
    fn copy_selected(&mut self, _action: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        let table = self.table_state.read(cx);

        // TODO: Tối ưu việc copy row thành một hàng của một bảng hơn thay vì "tab"
        if let Some(row_ix) = table.selected_row() {
            let delegate = table.delegate();
            let columns_count = delegate.columns_count(cx);
            let mut cells = Vec::with_capacity(columns_count);
            for col_ix in 0..columns_count {
                cells.push(delegate.cell_text(row_ix, col_ix, cx))
            }
            let text = cells.join("\t");
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }

        if let Some((row_ix, col_ix)) = table.selected_cell() {
            let text = table.delegate().cell_text(row_ix, col_ix, cx);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }
    }
}

impl Render for OutputPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let output = self.query.read(cx).output.clone();

        let content = match &output {
            OutputContent::Empty => div()
                .w_full()
                .flex_1()
                .p_3()
                .child(Label::new("Chưa có kết quả. Nhập SQL và nhấn Execute."))
                .into_any_element(),

            OutputContent::Execution { text } => div()
                .id("output-execution")
                .w_full()
                .flex_1()
                .p_3()
                .overflow_y_scrollbar()
                .child(Label::new(text.clone()).text_sm())
                .into_any_element(),

            OutputContent::Query { .. } => DataTable::new(&self.table_state)
                .stripe(true)
                .scrollbar_visible(true, true)
                .into_any_element(),

            OutputContent::Error(msg) => {
                let error = format!("Lỗi: {}", msg);
                println!("{}", error);

                div()
                    .id("output-error")
                    .w_full()
                    .flex_1()
                    .p_3()
                    .overflow_y_scrollbar()
                    .child(
                        Label::new(error).text_sm(), // .text_color(cx.theme()),
                    )
                    .into_any_element()
            }
        };

        v_flex()
            .id("output-panel")
            .key_context("datagrid")
            .on_action(cx.listener(Self::copy_selected))
            .w_full()
            .flex_1()
            .child(
                h_flex()
                    .w_full()
                    .px_3()
                    .py_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        Label::new("Output")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(content)
    }
}
