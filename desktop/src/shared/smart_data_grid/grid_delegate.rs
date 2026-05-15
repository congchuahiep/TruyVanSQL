use crate::shared::smart_data_grid::EditingState;

use super::grid_state::GridState;
use gpui::prelude::FluentBuilder;
use gpui::*;
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
        self.state.original_rows.len() // Tương lai sẽ cộng thêm dòng insert
    }

    fn column(&self, col_ix: usize, _: &App) -> GpuiColumn {
        self.cached_columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
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
        let is_deleted = self.state.pending_deletes.contains(&row_ix);

        let text: SharedString =
            if let Some(new_val) = self.state.pending_edits.get(&(row_ix, col_ix)) {
                new_val.clone().into()
            } else {
                self.state
                    .original_rows
                    .get(row_ix)
                    .and_then(|row| row.get(col_ix))
                    .cloned()
                    .unwrap_or_else(|| "".into())
            };

        if is_deleted {
            div()
                .p_neg_2()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .child(div().line_through().text_color(gpui::red()).child(text))
                .into_any_element()
        } else if is_edited {
            div()
                .p_neg_2()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .child(div().size_full().p_2().bg(gpui::rgb(0xfff085)).child(text))
                .into_any_element()
        } else {
            text.into_any_element()
        }
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        if let Some(new_val) = self.state.pending_edits.get(&(row_ix, col_ix)) {
            return new_val.clone();
        }

        self.state
            .original_rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .map(|val| val.to_string())
            .unwrap_or_default()
    }
}
