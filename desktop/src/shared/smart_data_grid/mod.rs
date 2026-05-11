mod data_changeset_builder;
pub mod delegate;
pub mod state;

use std::usize;

use assets::AppIcon;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::InputState;
use gpui_component::table::{DataTable, TableDelegate, TableEvent, TableState};
use gpui_component::{ActiveTheme, Disableable, h_flex, v_flex};
use thiserror::Error;

use crate::action::datagrid::{CancelEdit, CommitChanges, ConfirmEdit, CopyCell, StartEdit};
use crate::connection::model::DatabaseConnection;
use crate::shared::smart_data_grid::state::EditingState;
use engine::{Column, Row};

use delegate::GridDelegate;
use state::GridState;

fn validate_sql_type(text: &str, data_type: &str) -> bool {
    let data_type = data_type.to_uppercase();
    if data_type.contains("INT") || data_type.contains("BOOL") {
        text.parse::<i64>().is_ok()
    } else if data_type.contains("REAL")
        || data_type.contains("FLOAT")
        || data_type.contains("DOUBLE")
        || data_type.contains("DECIMAL")
        || data_type.contains("NUMERIC")
    {
        text.parse::<f64>().is_ok()
    } else {
        true
    }
}

/// View độc lập quản lý hiển thị và tương tác dữ liệu dạng bảng.
pub struct SmartDataGrid {
    pub connection: Entity<DatabaseConnection>,
    pub table: Entity<TableState<GridDelegate>>,
    pub cell_editor: Entity<InputState>,
    _blur_subscription: gpui::Subscription,
}

impl SmartDataGrid {
    pub fn new(
        connection: Entity<DatabaseConnection>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let cell_editor = cx.new(|cx| InputState::new(window, cx));

        let delegate = GridDelegate::new(GridState::new(), cell_editor.clone());
        let table = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .cell_selectable(true)
                .row_selectable(true)
        });

        let focus_handle = cell_editor.read(cx).focus_handle(cx);
        let blur_sub = cx.on_blur(&focus_handle, window, |this: &mut Self, window, cx| {
            match this.stage_cell_edit(window, cx) {
                Ok((row_ix, col_ix)) => {
                    this.table.update(cx, |table, cx| {
                        table.set_selected_cell(row_ix, col_ix, cx);
                    });
                }
                Err(error) => eprintln!("Blur error: {error}"),
            };
        });

        // Bắt sự kiện từ Table (ví dụ: Double Click để Edit)
        cx.subscribe_in(&table, window, Self::on_table_event)
            .detach();

        Self {
            connection,
            table,
            cell_editor,
            _blur_subscription: blur_sub,
        }
    }

    fn on_table_event(
        &mut self,
        table: &Entity<TableState<GridDelegate>>,
        event: &TableEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::DoubleClickedCell(row_ix, col_ix) => {
                self.activate_editor(*row_ix, *col_ix, window, cx);
            }
            TableEvent::SelectCell(row_ix, col_ix) => {
                // Focus lại editing cell khi giá trị edit hiện tại không hợp lệ
                if let Some(EditingState {
                    row,
                    col,
                    has_error,
                }) = self.table.read(cx).delegate().state.editing_state
                {
                    table.update(cx, |table, cx| {
                        table.clear_selection(cx);
                    });

                    if row == *row_ix && col == *col_ix && !has_error {
                        self.cell_editor.update(cx, |input, cx| {
                            input.focus(window, cx);
                        });
                    }
                }
            }
            _ => {}
        }
    }

    /// Kích hoạt chế độ chỉnh sửa cho một ô cụ thể
    fn activate_editor(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delegate = self.table.read(cx).delegate();
        let state = &delegate.state;

        if !state.is_editable()
            || state.editing_state.as_ref().is_some_and(
                |EditingState {
                     row,
                     col,
                     has_error,
                 }| (*row == row_ix && *col == col_ix) || *has_error,
            )
        {
            return;
        }

        let current_value = delegate.cell_text(row_ix, col_ix, cx);

        self.cell_editor.update(cx, |input, cx| {
            input.set_value(current_value, window, cx);
            input.focus(window, cx);
        });

        self.table.update(cx, |table, cx| {
            table.clear_selection(cx);
            table.delegate_mut().state.editing_state = Some(EditingState {
                row: row_ix,
                col: col_ix,
                has_error: false,
            });
        });
    }

    /// Đánh dấu cell hiện tại trong trạng thái chuẩn bị thay đổi, trước khi được lưu chính thức
    /// (commit) xuống dưới database
    fn stage_cell_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(usize, usize), StageError> {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            let (r, c) = match &delegate.state.editing_state {
                Some(s) => (s.row, s.col),
                None => return Err(StageError::NoActiveEdit),
            };

            let value = self.cell_editor.read(cx).value().to_string();
            let col_type = delegate.state.columns[c]
                .declared_type
                .clone()
                .unwrap_or_default();

            let original_value = delegate
                .state
                .original_rows
                .get(r)
                .and_then(|row| row.get(c))
                .map(|s| s.to_string())
                .unwrap_or_default();

            if value == original_value {
                delegate.state.pending_edits.remove(&(r, c));
                delegate.state.editing_state = None;
                cx.notify();
                return Ok((r, c));
            }

            if validate_sql_type(&value, &col_type) {
                delegate.state.pending_edits.insert((r, c), value);
                delegate.state.editing_state = None;
                cx.notify();
                Ok((r, c))
            } else {
                delegate.state.editing_state.as_mut().unwrap().has_error = true;
                self.cell_editor.update(cx, |ed, cx| ed.focus(window, cx));
                cx.notify();
                Err(StageError::InvalidData(format!(
                    "Giá trị '{}' không đúng định dạng {}",
                    value, col_type
                )))
            }
        })
    }

    /// Cập nhật dữ liệu gốc cho Grid
    pub fn set_data(&mut self, columns: Vec<Column>, rows: Vec<Row>, cx: &mut Context<Self>) {
        let cached_rows: Vec<Vec<SharedString>> = rows
            .into_iter()
            .map(|row| {
                row.values
                    .into_iter()
                    .map(|val| match val {
                        Some(v) => v.to_string().into(),
                        None => "NULL".into(),
                    })
                    .collect()
            })
            .collect();

        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.state.columns = columns;
            delegate.state.original_rows = cached_rows;
            delegate.state.pending_edits.clear();
            delegate.state.pending_deletes.clear();
            delegate.state.pending_inserts.clear();

            // Cập nhật lại cached_columns trong Delegate
            *delegate = GridDelegate::new(delegate.state.clone(), self.cell_editor.clone());
            table.refresh(cx);
        });
    }

    /// Cấu hình siêu dữ liệu để Grid biết nó có thể Edit được không
    pub fn set_metadata(
        &mut self,
        source_table: Option<String>,
        primary_keys: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.state.source_table = source_table;
            delegate.state.primary_keys = primary_keys;
            cx.notify();
        });
    }

    fn on_refresh(&mut self, _: &ClickEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        println!("SmartDataGrid: Đã bấm nút Refresh");
    }

    /// Copy cell hoặc row đang selected vào clipboard.
    /// - Nếu có cell selected → copy cell value
    /// - Nếu có row selected → copy cả row (tab-separated)
    fn on_copy_cell(&mut self, _: &CopyCell, _window: &mut Window, cx: &mut Context<Self>) {
        let table = self.table.read(cx);

        // TODO: Triển khai khả năng copy cột
        // TODO: Tối ưu việc copy row theo nhiều loại định dạng khác nhau: CSV, TSV, JSON,...
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

    fn on_commit_changes(
        &mut self,
        _: &CommitChanges,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = &self.table.read(cx).delegate().state;
        let builder = data_changeset_builder::DataChangesetBuilder::new(state);

        match builder.build_changeset() {
            Ok(changeset) => {
                // Lấy client từ DatabaseConnection
                let connection = self.connection.read(cx);
                if let Some(client) = &connection.client {
                    let script = client.generate_changeset_script(&changeset);
                    println!("SQL Script generated by Engine:\n{}", script);
                }
            }
            Err(error) => eprintln!("Builder error: {}", error),
        }
    }

    fn on_start_edit(&mut self, _: &StartEdit, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((r, c)) = self.table.read(cx).selected_cell() {
            self.activate_editor(r, c, window, cx);
        }
    }

    fn on_confirm_edit(&mut self, _: &ConfirmEdit, window: &mut Window, cx: &mut Context<Self>) {
        match self.stage_cell_edit(window, cx) {
            Ok((row_ix, col_ix)) => {
                self.table.update(cx, |table, cx| {
                    table.focus_handle(cx).focus(window, cx);
                    table.set_selected_cell(row_ix, col_ix, cx);
                });
            }
            Err(error) => eprintln!("Enter key error: {error}"),
        };
    }

    fn on_cancel_edit(&mut self, _: &CancelEdit, window: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            if let Some(EditingState { row, col, .. }) = table.delegate().state.editing_state {
                table.set_selected_cell(row, col, cx);
            }

            table.focus_handle(cx).focus(window, cx);
            table.delegate_mut().state.editing_state = None;
        });
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let delegate = self.table.read(cx).delegate();
        let state = &delegate.state;

        let is_editable = state.is_editable();
        let has_changes = state.has_pending_changes();
        let is_loading = state.is_loading;

        h_flex()
            .w_full()
            .p_1()
            .gap_px()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                Button::new("btn-refresh")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::Refresh)
                    .disabled(is_loading)
                    .on_click(cx.listener(Self::on_refresh)),
            )
            .child(div().w_px().h_4().mx_px().bg(cx.theme().border))
            .child(
                Button::new("btn-add-row")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::Plus)
                    .disabled(!is_editable || is_loading),
            )
            .child(
                Button::new("btn-delete-row")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::Minus)
                    .disabled(!is_editable || is_loading),
            )
            .child(div().w_px().h_4().mx_px().bg(cx.theme().border))
            .child(
                Button::new("btn-submit-changes")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::Check)
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(CommitChanges), cx);
                    })
                    .disabled(!has_changes || is_loading),
            )
            .child(
                Button::new("btn-cancel")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::X)
                    .disabled(!has_changes || is_loading),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Rows: {}", state.original_rows.len())),
            )
    }
}

impl Render for SmartDataGrid {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("data-grid-container")
            .on_action(cx.listener(Self::on_commit_changes))
            .on_action(cx.listener(Self::on_copy_cell))
            .size_full()
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .key_context("data-grid")
                    .on_action(cx.listener(Self::on_confirm_edit))
                    .on_action(cx.listener(Self::on_cancel_edit))
                    .on_action(cx.listener(Self::on_start_edit))
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(false)
                            .scrollbar_visible(true, true),
                    ),
            )
    }
}

#[derive(Error, Debug)]
pub enum StageError {
    #[error("Dữ liệu không hợp lệ: {0}")]
    InvalidData(String),

    #[error("Không có phiên chỉnh sửa nào đang hoạt động")]
    NoActiveEdit,
}
