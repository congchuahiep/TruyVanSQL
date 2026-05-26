use engine::Column;
use gpui::SharedString;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct EditingState {
    pub row: usize,
    pub col: usize,
    pub has_error: bool,
}

#[derive(Clone)]
pub struct GridState {
    pub columns: Vec<Column>,
    pub original_rows: Vec<Vec<SharedString>>,

    pub source_table: Option<SharedString>,
    pub primary_keys: Vec<String>,

    pub pending_edits: HashMap<(usize, usize), String>,
    pub pending_deletes: HashSet<usize>,
    pub pending_inserts: Vec<Vec<String>>,

    pub limit: usize,
    pub offset: usize,
    pub total_rows: Option<usize>,
    pub error: Option<SharedString>,
    pub is_loading: bool,

    pub editing_state: Option<EditingState>,
}

impl GridState {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            original_rows: Vec::new(),
            source_table: None,
            primary_keys: Vec::new(),
            pending_edits: HashMap::new(),
            pending_deletes: HashSet::new(),
            pending_inserts: Vec::new(),
            limit: 1000,
            offset: 0,
            total_rows: None,
            error: None,
            is_loading: false,
            editing_state: None,
        }
    }

    /// Số lượng original rows (không tính pending_inserts)
    pub fn original_len(&self) -> usize {
        self.original_rows.len()
    }

    /// Tổng số rows hiển thị trên grid
    pub fn total_len(&self) -> usize {
        self.original_rows.len() + self.pending_inserts.len()
    }

    /// Đổi row index toàn cục → index trong pending_inserts
    pub fn insert_index(&self, row_ix: usize) -> usize {
        row_ix - self.original_len()
    }

    /// Kiểm tra xem grid có thể chỉnh sửa không (có source_table và primary_keys)
    pub fn is_editable(&self) -> bool {
        self.source_table.is_some() && !self.primary_keys.is_empty()
    }

    /// Row này có bị đánh dấu xóa không?
    pub fn is_deleted_row(&self, row_ix: usize) -> bool {
        row_ix < self.original_rows.len() && self.pending_deletes.contains(&row_ix)
    }

    /// Row này có phải là pending insert không?
    pub fn is_inserted_row(&self, row_ix: usize) -> bool {
        row_ix >= self.original_rows.len()
    }

    /// Kiểm tra xem sự thay đổi vào database không
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_edits.is_empty()
            || !self.pending_deletes.is_empty()
            || !self.pending_inserts.is_empty()
    }

    /// Lấy text của một cell (ưu tiên pending_edits > pending_inserts > original)
    pub fn cell_value(&self, row_ix: usize, col_ix: usize) -> String {
        // Pending edit (cả original lẫn inserted đều có thể có)
        if let Some(val) = self.pending_edits.get(&(row_ix, col_ix)) {
            return val.clone();
        }

        if self.is_inserted_row(row_ix) {
            let ix = row_ix - self.original_len();
            return self
                .pending_inserts
                .get(ix)
                .and_then(|row| row.get(col_ix))
                .cloned()
                .unwrap_or_default();
        }

        self.original_rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}
