use crate::shared::smart_data_grid::EditingState;

use super::grid_state::GridState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::input::Input;
use gpui_component::input::InputState;
use gpui_component::table::{Column as GpuiColumn, TableDelegate, TableState};

#[derive(Clone)]
pub struct GridDelegate {
    pub state: GridState,
    pub cached_columns: Vec<GpuiColumn>,
    pub cell_editor: Entity<InputState>,
}

impl GridDelegate {
    pub fn new(state: GridState, cell_editor: Entity<InputState>) -> Self {
        let cached_columns = state
            .columns
            .iter()
            .map(|col| {
                GpuiColumn::new(
                    SharedString::from(col.name.clone()),
                    SharedString::from(col.name.clone()),
                )
                .width(150.0)
            })
            .collect();

        Self {
            state,
            cached_columns,
            cell_editor,
        }
    }
}

impl TableDelegate for GridDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.cached_columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.state.original_rows.len() + self.state.pending_inserts.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> GpuiColumn {
        self.cached_columns[col_ix].clone()
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        let is_inserted = self.state.is_inserted_row(row_ix);
        let is_deleted = self.state.is_deleted_row(row_ix);
        let is_stripe = row_ix % 2 != 0;

        div()
            .id(("row", row_ix))
            .relative()
            .when(is_deleted, |this| {
                this.child(div().absolute().inset_0().bg(cx.theme().danger))
                    .line_through()
                    .text_color(cx.theme().danger_foreground)
            })
            .when(is_inserted, |this| {
                this.child(div().absolute().inset_0().bg(cx.theme().success))
            })
            .when(is_stripe, |this| this.bg(cx.theme().table_even))
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if let Some(EditingState {
            row,
            col,
            has_error,
        }) = self.state.editing_state
            && row == row_ix
            && col == col_ix
        {
            let wrapper = div().px_neg_2().py_neg_1().size_full();

            return wrapper
                .key_context("cell-editor")
                .child(
                    Input::new(&self.cell_editor)
                        .appearance(false)
                        .px_2()
                        .size_full()
                        .text_base()
                        .border_2()
                        .border_color(rgb(0x2b7fff))
                        .when(has_error, |this| {
                            this.border_color(gpui::red()).bg(rgb(0xffc9c9))
                        }),
                )
                .into_any_element();
        }

        let is_edited = self.state.pending_edits.contains_key(&(row_ix, col_ix));

        let original_len = self.state.original_rows.len();
        let text: SharedString =
            if let Some(new_val) = self.state.pending_edits.get(&(row_ix, col_ix)) {
                new_val.clone().into()
            } else if row_ix >= original_len {
                let insert_ix = row_ix - original_len;
                self.state
                    .pending_inserts
                    .get(insert_ix)
                    .and_then(|row| row.get(col_ix))
                    .cloned()
                    .map(SharedString::from)
                    .unwrap_or_default()
            } else {
                self.state
                    .original_rows
                    .get(row_ix)
                    .and_then(|row| row.get(col_ix))
                    .cloned()
                    .unwrap_or_default()
            };

        let outer = div().p_neg_2().w_full().h_full().flex().items_center();

        let mut inner = div()
            .size_full()
            .border_r_1()
            .border_color(cx.theme().border);

        if is_edited {
            inner = inner.p_2().bg(cx.theme().warning);
        } else {
            inner = inner.p_2();
        }

        outer.child(inner.child(text)).into_any_element()
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div().into_any_element()
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        // Kiểm tra pending edits trước (áp dụng cho cả original lẫn inserted)
        if let Some(new_val) = self.state.pending_edits.get(&(row_ix, col_ix)) {
            return new_val.clone();
        }
        let original_len = self.state.original_rows.len();
        // Nếu là inserted row (nằm ngoài phạm vi original)
        if row_ix >= original_len {
            let insert_ix = row_ix - original_len;
            return self
                .state
                .pending_inserts
                .get(insert_ix)
                .and_then(|row| row.get(col_ix))
                .cloned()
                .unwrap_or_default();
        }
        // Original row
        self.state
            .original_rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .map(|val| val.to_string())
            .unwrap_or_default()
    }
}
