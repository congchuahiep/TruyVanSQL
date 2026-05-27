use super::{DataChangesetBuilder, EditingState, GridDelegate, GridState, StageError};
use assets::AppIcon;
use engine::{Column, Row, SqlClient};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::input::InputState;
use gpui_component::table::{DataTable, TableDelegate, TableEvent, TableState};
use gpui_component::{ActiveTheme, Disableable, Icon, Sizable, h_flex, v_flex};

/// View độc lập quản lý hiển thị và tương tác dữ liệu dạng bảng.
pub struct SmartDataGrid {
    pub client: SqlClient,
    pub table: Entity<TableState<GridDelegate>>,
    pub cell_editor: Entity<InputState>,
    focus_handle: FocusHandle,
    _blur_subscription: gpui::Subscription,
}

impl SmartDataGrid {
    pub fn new(client: SqlClient, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let cell_editor = cx.new(|cx| InputState::new(window, cx));

        let delegate = GridDelegate::new(GridState::new(), cell_editor.clone());
        let table = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .row_header(false)
                .cell_selectable(true)
                .row_selectable(true)
        });

        let cell_editor_focus_handle = cell_editor.read(cx).focus_handle(cx);
        let blur_sub = cx.on_blur(
            &cell_editor_focus_handle,
            window,
            |this: &mut Self, window, cx| {
                match this.stage_cell_edit(window, cx) {
                    Ok((row_ix, col_ix)) => {
                        this.table.update(cx, |table, cx| {
                            table.set_selected_cell(row_ix, col_ix, cx);
                        });
                    }
                    Err(error) => eprintln!("Blur error: {error}"),
                };
            },
        );

        let focus_handle = cx.focus_handle();

        // Bắt sự kiện từ Table (ví dụ: Double Click để Edit)
        cx.subscribe_in(&table, window, Self::on_table_event)
            .detach();

        Self {
            client,
            table,
            cell_editor,
            focus_handle,
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

        let current_value = delegate.state.cell_value(row_ix, col_ix);
        let current_value = match current_value {
            Some(val) => val.to_string(),
            None => String::new(),
        };

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
            let state = &mut delegate.state;

            let (r, c) = match &state.editing_state {
                Some(s) => (s.row, s.col),
                None => return Err(StageError::NoActiveEdit),
            };

            let text = self.cell_editor.read(cx).value().to_string();
            let col_type = state.columns[c].declared_type.clone().unwrap_or_default();
            let data_type_category = self.client.data_type_categorize(&col_type);

            let new_value: Option<SharedString> = match text.is_empty() {
                true => data_type_category
                    .allows_empty_string()
                    .then_some(SharedString::from("")),
                false => {
                    if !data_type_category.validate(&text) {
                        state.editing_state.as_mut().unwrap().has_error = true;
                        self.cell_editor.update(cx, |ed, cx| ed.focus(window, cx));
                        cx.notify();
                        return Err(StageError::InvalidData(format!(
                            "Giá trị '{}' không đúng định dạng {}",
                            text, col_type
                        )));
                    }
                    Some(SharedString::from(text))
                }
            };

            match state.is_inserted_row(r) {
                true => {
                    let insert_index = state.insert_index(r);
                    state
                        .pending_inserts
                        .get_mut(insert_index)
                        .and_then(|row| row.get_mut(c))
                        .map(|cell| *cell = new_value);
                }
                false => {
                    let original = state.cell_original_value(r, c);
                    match new_value == original {
                        true => state.pending_edits.remove(&(r, c)),
                        false => state.pending_edits.insert((r, c), new_value),
                    };
                }
            }

            state.editing_state = None;
            cx.notify();
            Ok((r, c))
        })
    }

    /// Cập nhật dữ liệu gốc cho Grid
    pub fn set_data(&mut self, columns: Vec<Column>, rows: Vec<Row>, cx: &mut Context<Self>) {
        let cached_rows: Vec<Vec<Option<SharedString>>> = rows
            .into_iter()
            .map(|row| {
                row.values
                    .into_iter()
                    .map(|val| match val {
                        Some(v) => Some(v.to_string().into()),
                        None => None,
                    })
                    .collect()
            })
            .collect();

        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.state.original_rows = cached_rows;
            delegate.state.columns = columns;
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
        source_table: Option<SharedString>,
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

    /// Refresh lại 
    fn on_refresh(&mut self, _: &ClickEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        println!("SmartDataGrid: Đã bấm nút Refresh");
    }

    /// Copy cell hoặc row đang selected vào clipboard.
    /// - Nếu có cell selected → copy cell value
    /// - Nếu có row selected → copy cả row (tab-separated)
    fn on_copy_cell(
        &mut self,
        _: &crate::action::datagrid::CopyCell,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    /// Lưu các thay đổi xuống dưới database thực tế
    fn on_commit_changes(
        &mut self,
        _: &crate::action::datagrid::CommitChanges,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = &self.table.read(cx).delegate().state;
        let builder = DataChangesetBuilder::new(state);

        match builder.build_changeset() {
            Ok(changeset) => {
                let script = self.client.generate_changeset_script(&changeset);
                println!("SQL Script generated by Engine:\n{}", script);
            }
            Err(error) => eprintln!("Builder error: {}", error),
        }
    }

    fn on_start_edit(
        &mut self,
        _: &crate::action::datagrid::StartEdit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((r, c)) = self.table.read(cx).selected_cell() {
            self.activate_editor(r, c, window, cx);
        }
    }

    fn on_confirm_edit(
        &mut self,
        _: &crate::action::datagrid::ConfirmEdit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    fn on_cancel_edit(
        &mut self,
        _: &crate::action::datagrid::CancelEdit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.table.update(cx, |table, cx| {
            if let Some(EditingState { row, col, .. }) = table.delegate().state.editing_state {
                table.set_selected_cell(row, col, cx);
            }

            table.focus_handle(cx).focus(window, cx);
            table.delegate_mut().state.editing_state = None;
        });
    }

    /// Xóa dòng đã chọn, hoặc dòng chứa ô đã chọn nếu không có dòng nào được chọn
    fn on_delete_row(
        &mut self,
        _: &crate::action::datagrid::DeleteRow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.table.update(cx, |table, cx| {
            let selected_row = if let Some(row_ix) = table.selected_row() {
                row_ix
            } else if let Some((row_ix, _)) = table.selected_cell() {
                row_ix
            } else {
                return; // Không làm gì cả nếu không select được row nào
            };

            let delegate = table.delegate_mut();
            let state = &mut delegate.state;
            if state.is_inserted_row(selected_row) {
                let ix = state.insert_index(selected_row);
                state.pending_inserts.remove(ix);
            } else {
                if state.pending_deletes.contains(&selected_row) {
                    state.pending_deletes.remove(&selected_row);
                } else {
                    state.pending_deletes.insert(selected_row);
                }
            }
            cx.notify();
        });
    }

    /// Thêm một dòng mới vào cuối bảng
    fn on_add_row(
        &mut self,
        _: &crate::action::datagrid::AddRow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            let col_count = delegate.state.columns.len();
            if col_count == 0 {
                return;
            }
            delegate.state.pending_inserts.push(vec![None; col_count]);
            cx.notify();
        });
    }

    fn on_discard_changes(
        &mut self,
        _: &crate::action::datagrid::DiscardChanges,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.state.pending_edits.clear();
            delegate.state.pending_deletes.clear();
            delegate.state.pending_inserts.clear();
            delegate.state.editing_state = None;
            cx.notify();
        });
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let delegate = self.table.read(cx).delegate();
        let state = &delegate.state;

        let is_editable = state.is_editable();
        let has_changes = state.has_pending_changes();
        let is_loading = state.is_loading;

        let commit_changes_button = ButtonCustomVariant::new(cx)
            .color(cx.theme().green)
            .hover(cx.theme().green_light)
            .active(cx.theme().green_light);

        let discard_change_button = ButtonCustomVariant::new(cx)
            .color(cx.theme().red)
            .hover(cx.theme().red_light)
            .active(cx.theme().red_light);

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
                    .disabled(!is_editable || is_loading)
                    .on_click({
                        let focus_handle = self.focus_handle.clone();
                        move |_, window, cx| {
                            focus_handle.dispatch_action(
                                &crate::action::datagrid::AddRow,
                                window,
                                cx,
                            );
                        }
                    }),
            )
            .child(
                Button::new("btn-delete-row")
                    .ghost()
                    .size_6()
                    .cursor_pointer()
                    .icon(AppIcon::Minus)
                    .disabled(!is_editable || is_loading)
                    .on_click({
                        let focus_handle = self.focus_handle.clone();
                        move |_, window, cx| {
                            focus_handle.dispatch_action(
                                &crate::action::datagrid::DeleteRow,
                                window,
                                cx,
                            );
                        }
                    }),
            )
            .child(div().w_px().h_4().mx_px().bg(cx.theme().border))
            .child(
                Button::new("btn-submit-changes")
                    .outline()
                    .border_0()
                    .custom(commit_changes_button)
                    .size_6()
                    .cursor_pointer()
                    .icon(Icon::new(AppIcon::Check))
                    .on_click({
                        let focus_handle = self.focus_handle.clone();
                        move |_, window, cx| {
                            focus_handle.dispatch_action(
                                &crate::action::datagrid::CommitChanges,
                                window,
                                cx,
                            );
                        }
                    })
                    .disabled(!has_changes || is_loading),
            )
            .child(
                Button::new("btn-cancel")
                    .outline()
                    .border_0()
                    .custom(discard_change_button)
                    .size_6()
                    .cursor_pointer()
                    .icon(Icon::new(AppIcon::X))
                    .disabled(!has_changes || is_loading)
                    .on_click({
                        let focus_handle = self.focus_handle.clone();
                        move |_, window, cx| {
                            focus_handle.dispatch_action(
                                &crate::action::datagrid::DiscardChanges,
                                window,
                                cx,
                            );
                        }
                    }),
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
        let state = self.table.read(cx).delegate().state.clone();

        v_flex()
            .key_context("data-grid-container")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_commit_changes))
            .on_action(cx.listener(Self::on_copy_cell))
            .on_action(cx.listener(Self::on_add_row))
            .on_action(cx.listener(Self::on_delete_row))
            .on_action(cx.listener(Self::on_discard_changes))
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
                    .font_family(cx.theme().mono_font_family.clone())
                    // Hiển thị error view
                    .when_else(
                        state.error.is_fatal(),
                        |this| {
                            this.child(
                                v_flex().size_full().items_center().justify_center().child(
                                    v_flex()
                                        .items_center()
                                        .gap_2()
                                        .max_w_128()
                                        .px_12()
                                        .child(
                                            div()
                                                .line_height(px(24.))
                                                .text_xl()
                                                .text_color(cx.theme().danger_foreground)
                                                .child(
                                                    Icon::new(AppIcon::TriangleWarningFill)
                                                        .with_size(px(32.)),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(state.error.message()),
                                        ),
                                ),
                            )
                        },
                        |this| {
                            this.child(
                                DataTable::new(&self.table)
                                    .bordered(false)
                                    .scrollbar_visible(true, true),
                            )
                        },
                    ),
            )
    }
}
