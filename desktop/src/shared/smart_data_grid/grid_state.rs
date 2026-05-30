use super::GridFetchState;
use engine::Column;
use gpui::SharedString;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct GridState {
    pub columns: Vec<Column>,
    pub original_rows: Vec<Vec<Option<SharedString>>>,

    pub data_source: GridDataSource,
    pub primary_keys: Vec<String>,

    pub pending_edits: HashMap<(usize, usize), Option<SharedString>>,
    pub pending_deletes: HashSet<usize>,
    pub pending_inserts: Vec<Vec<Option<SharedString>>>,

    pub total_rows: Option<usize>,

    /// Trạng thái fetch data, dùng để xác định có thể hiển thị data từ GridState không?
    ///
    /// - [`GridFetchState::Idle`]: Không có data đang được fetch
    /// - [`GridFetchState::Loading`]: Đang fetch data
    /// - [`GridFetchState::Loaded`]: Fetch data thành công
    /// - [`GridFetchState::Error`]: Fetch data thất bại
    pub fetch_state: GridFetchState,

    pub editing_state: Option<EditingState>,
}

impl GridState {
    pub fn new(data_source: GridDataSource) -> Self {
        Self {
            columns: Vec::new(),
            original_rows: Vec::new(),
            data_source,
            primary_keys: Vec::new(),
            pending_edits: HashMap::new(),
            pending_deletes: HashSet::new(),
            pending_inserts: Vec::new(),
            total_rows: None,
            fetch_state: GridFetchState::Idle,
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

    /// Lấy tên bảng nguồn của grid (nếu có)
    pub fn source_table(&self) -> Option<SharedString> {
        match &self.data_source {
            GridDataSource::Table { source_table, .. } => Some(source_table.clone()),
            GridDataSource::Query { source_table, .. } => source_table.clone(),
        }
    }

    /// Kiểm tra xem có thể phân trang hay không
    pub fn can_paginate(&self) -> bool {
        matches!(self.data_source, GridDataSource::Table { .. })
    }

    /// Kiểm tra xem có thể thêm dòng mới vào grid hay không
    ///
    /// TODO: Query hoàn toàn có thể insert theo một trường hợp nào đó
    pub fn can_insert(&self) -> bool {
        matches!(self.data_source, GridDataSource::Table { .. })
    }

    /// Kiểm tra xem grid có thể chỉnh sửa không (có source_table và primary_keys)
    pub fn can_edit(&self) -> bool {
        match &self.data_source {
            GridDataSource::Table { .. } => !self.primary_keys.is_empty(),
            GridDataSource::Query {
                source_table: Some(_),
                ..
            } => !self.primary_keys.is_empty(),
            GridDataSource::Query {
                source_table: None, ..
            } => false,
        }
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
    pub fn cell_value(&self, row_ix: usize, col_ix: usize) -> Option<SharedString> {
        // Pending edit (cả original lẫn inserted đều có thể có)
        if let Some(editing_value) = self.pending_edits.get(&(row_ix, col_ix)) {
            return editing_value.clone();
        }

        if self.is_inserted_row(row_ix) {
            let ix = row_ix - self.original_len();
            return self
                .pending_inserts
                .get(ix)
                .and_then(|row| row.get(col_ix))
                .cloned()
                .flatten();
        }

        self.cell_original_value(row_ix, col_ix)
    }

    /// Lấy giá trị gốc của một cell (không bao gồm pending edits)
    pub fn cell_original_value(&self, row_ix: usize, col_ix: usize) -> Option<SharedString> {
        self.original_rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .flatten()
    }
}

#[derive(Clone)]
pub struct EditingState {
    pub row: usize,
    pub col: usize,
    pub has_error: bool,
}

#[derive(Clone, Debug)]
pub enum GridDataSource {
    /// TableViewer: data từ 1 bảng, có phân trang, thêm dòng
    Table {
        source_table: SharedString,
        limit: usize,
        offset: usize,
    },
    /// SQL Editor: data từ query tùy ý, không phân trang
    Query {
        source_table: Option<SharedString>,
        query: SharedString,
    },
}
