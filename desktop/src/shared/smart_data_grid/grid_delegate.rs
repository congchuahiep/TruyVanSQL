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

        let cell_val = self.state.cell_value(row_ix, col_ix);
        let is_null = cell_val.is_none();
        let is_edited = self.state.pending_edits.contains_key(&(row_ix, col_ix));

        let outer = div().p_neg_2().w_full().h_full().flex().items_center();

        let mut inner = div()
            .p_2()
            .size_full()
            .border_r_1()
            .border_color(cx.theme().border);

        match (is_edited, is_null) {
            (true, true) => {
                // Edited → NULL: yellow bg + italic "NULL"
                inner = inner
                    .bg(cx.theme().warning)
                    .text_color(cx.theme().muted_foreground)
                    .italic()
                    .child("NULL".to_string());
            }
            (true, false) => {
                // Edited → value: yellow bg + value
                inner = inner.bg(cx.theme().warning).child(cell_val.unwrap());
            }
            (false, true) => {
                // Original NULL: italic "NULL" muted
                inner = inner
                    .text_color(cx.theme().muted_foreground)
                    .italic()
                    .child("NULL".to_string());
            }
            (false, false) => {
                // Original value: normal text
                inner = inner.child(cell_val.unwrap());
            }
        }

        outer.child(inner).into_any_element()
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div().into_any_element()
    }

    /// Lấy text cho clipboard. NULL → "NULL" string.
    ///
    /// Không khuyến khích sử dụng hàm này vì nó không xử lý giá trị NULL theo đúng cách, thay vào
    /// đó hãy sử dụng [`GridState.cell_value`]
    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        match self.state.cell_value(row_ix, col_ix) {
            Some(val) => val.to_string(),
            None => "NULL".to_string(),
        }
    }
}
