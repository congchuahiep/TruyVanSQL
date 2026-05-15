use crate::shared::smart_data_grid::grid_state::GridState;
use engine::{ColumnData, DataChangeset, RowDelete, RowUpdate};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataChangesetError {
    #[error("Không tìm thấy tên bảng gốc")]
    TableUndefinded,
    #[error("Không thể tạo script vì {0} không có khoá chính")]
    PrimaryKeyNotFound(String),
}

pub struct DataChangesetBuilder<'a> {
    state: &'a GridState,
}

impl<'a> DataChangesetBuilder<'a> {
    pub fn new(state: &'a GridState) -> Self {
        Self { state }
    }

    pub fn build_changeset(&self) -> Result<DataChangeset, DataChangesetError> {
        let table_name = self
            .state
            .source_table
            .as_deref()
            .ok_or(DataChangesetError::TableUndefinded)?
            .to_string();

        if self.state.primary_keys.is_empty() {
            return Err(DataChangesetError::PrimaryKeyNotFound(table_name));
        }

        let mut updates = Vec::new();
        let mut edits_by_row: HashMap<usize, Vec<(usize, String)>> = HashMap::new();

        for (&(row, col), value) in &self.state.pending_edits {
            edits_by_row
                .entry(row)
                .or_default()
                .push((col, value.clone()));
        }

        for (row_ix, row_edits) in edits_by_row {
            let mut changes = Vec::new();
            for (col_ix, value) in row_edits {
                let col = &self.state.columns[col_ix];
                changes.push(ColumnData {
                    column_name: col.name.clone(),
                    value,
                    data_type: col.declared_type.clone().unwrap_or_default(),
                });
            }
            updates.push(RowUpdate {
                pk_conditions: self.get_pk_conditions(row_ix),
                changes,
            });
        }

        let mut deletes = Vec::new();
        for &row_ix in &self.state.pending_deletes {
            deletes.push(RowDelete {
                pk_conditions: self.get_pk_conditions(row_ix),
            });
        }

        Ok(DataChangeset {
            table_name,
            updates,
            deletes,
        })
    }

    fn get_pk_conditions(&self, row_ix: usize) -> Vec<ColumnData> {
        self.state
            .primary_keys
            .iter()
            .map(|pk_name| {
                let pk_col_ix = self
                    .state
                    .columns
                    .iter()
                    .position(|c| &c.name == pk_name)
                    .unwrap();
                let original_val = self.state.original_rows[row_ix][pk_col_ix].to_string();
                let col_type = self.state.columns[pk_col_ix]
                    .declared_type
                    .clone()
                    .unwrap_or_default();
                ColumnData {
                    column_name: pk_name.clone(),
                    value: original_val,
                    data_type: col_type,
                }
            })
            .collect()
    }
}
