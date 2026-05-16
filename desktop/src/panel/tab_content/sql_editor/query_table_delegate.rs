use gpui::prelude::*;
use gpui::*;
use gpui_component::table::{Column as GpuiColumn, TableDelegate, TableState};

/// Adapter giữa engine::QueryResult và gpui_component::DataTable.
pub struct QueryTableDelegate {
    pub columns: Vec<GpuiColumn>,
    pub rows: Vec<Vec<SharedString>>,
}

impl QueryTableDelegate {
    pub fn new(engine_columns: &[engine::Column], engine_rows: &[engine::Row]) -> Self {
        let columns: Vec<GpuiColumn> = engine_columns
            .iter()
            .map(|col| GpuiColumn::new(&col.name, col.name.clone()).width(150.0))
            .collect();

        let rows: Vec<Vec<SharedString>> = engine_rows
            .iter()
            .map(|row| {
                row.values
                    .iter()
                    .map(|val| match val {
                        Some(v) => v.to_string().into(),
                        None => "NULL".into(),
                    })
                    .collect()
            })
            .collect();

        Self { columns, rows }
    }

    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}

impl TableDelegate for QueryTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> GpuiColumn {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        self.rows[row_ix][col_ix].clone().into_any_element()
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        self.rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}
