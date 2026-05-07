use crate::shared::smart_data_grid::state::{self, GridState};
use std::{collections::HashMap, hash::Hash, string};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GenerateScriptError {
    #[error("Không tìm thấy tên bảng gốc")]
    TableUndefinded,

    #[error("Không thể tạo script vì {0} không có khoá chính")]
    PrimaryKeyNotFound(String),
}

pub struct ScriptGenerator<'a> {
    state: &'a GridState,
    table_name: &'a str,
    script: String,
}

impl<'a> ScriptGenerator<'a> {
    /// Khởi tạo một `ScriptGenerator` mới từ trạng thái hiện tại của GridState.
    ///
    /// Hàm này sẽ bóc tách các thông tin cần thiết và đảm bảo lưới dữ liệu
    /// đã sẵn sàng để tạo script.
    ///
    /// # Errors
    ///
    /// Hàm sẽ trả về [`GenerateScriptError`] trong các trường hợp:
    /// - `TableUndefinded`: Nếu `state` chưa được gán `source_table`.
    /// - `PrimaryKeyNotFound`: Nếu mảng `primary_keys` của `state` bị rỗng.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut state = GridState::new();
    /// state.source_table = Some("users".to_string());
    /// state.primary_keys = vec!["id".to_string()];
    ///
    /// // Khởi tạo thành công
    /// let generator = ScriptGenerator::try_new(&state);
    /// assert!(generator.is_ok());
    /// ```
    pub fn try_new(state: &'a GridState) -> Result<Self, GenerateScriptError> {
        let table_name = state
            .source_table
            .as_deref()
            .ok_or(GenerateScriptError::TableUndefinded)?;

        if state.primary_keys.is_empty() {
            return Err(GenerateScriptError::PrimaryKeyNotFound(
                table_name.to_string(),
            ));
        }

        Ok(Self {
            state,
            table_name,
            script: String::new(),
        })
    }

    pub fn generate(mut self) -> String {
        self.generate_update();
        self.generate_delete();

        self.script
    }

    fn generate_update(&mut self) {
        let mut edits_by_row: HashMap<usize, Vec<(usize, String)>> = HashMap::new();

        for (&(row, col), value) in &self.state.pending_edits {
            edits_by_row
                .entry(row)
                .or_default()
                .push((col, value.clone()));
        }

        for (row_ix, row_edits) in edits_by_row {
            let set_clause = row_edits
                .iter()
                .map(|(col_ix, value)| {
                    let col = &self.state.columns[*col_ix];
                    let col_name = &col.name;
                    let col_type = col.declared_type.as_deref().unwrap_or("");
                    format!(
                        "\"{}\" = {}",
                        col_name,
                        Self::format_sql_value(value, col_type)
                    )
                })
                .collect::<Vec<String>>()
                .join(", ");

            let where_clause = self.generate_where_clause(row_ix);

            self.script.push_str(&format!(
                "UPDATE \"{}\" SET {} WHERE {};\n",
                self.table_name, set_clause, where_clause
            ));
        }
    }

    fn generate_delete(&mut self) {
        for &row_ix in &self.state.pending_deletes {
            let where_clause = self.generate_where_clause(row_ix);
            self.script.push_str(&format!(
                "DELETE FROM \"{}\" WHERE {};\n",
                self.table_name, where_clause
            ));
        }
    }

    fn generate_where_clause(&mut self, row_ix: usize) -> String {
        self.state
            .primary_keys
            .iter()
            .map(|pk_name| {
                // Tìm vị trí của cột PK trong danh sách cột hiện tại
                let pk_col_ix = self
                    .state
                    .columns
                    .iter()
                    .position(|c| &c.name == pk_name)
                    .unwrap();
                let original_val = &self.state.original_rows[row_ix][pk_col_ix];
                let col_type = self.state.columns[pk_col_ix]
                    .declared_type
                    .as_deref()
                    .unwrap_or("");

                format!(
                    "\"{}\" = {}",
                    pk_name,
                    Self::format_sql_value(original_val, col_type)
                )
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }

    /// Định dạng giá trị chuẩn SQL
    fn format_sql_value(val: &str, declared_type: &str) -> String {
        if val == "NULL" {
            "NULL".into()
        } else {
            let dt = declared_type.to_uppercase();
            // Nếu là kiểu số, không bọc nháy (với điều kiện chuỗi thực sự là số)
            if (dt.contains("INT") || dt.contains("REAL") || dt.contains("FLOAT"))
                && val.chars().all(|c| c.is_digit(10) || c == '.' || c == '-')
            {
                val.into()
            } else {
                // Kiểu chuỗi: escape nháy đơn và bọc nháy
                format!("'{}'", val.replace("'", "''"))
            }
        }
    }
}
