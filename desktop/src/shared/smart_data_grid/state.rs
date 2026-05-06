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

    pub source_table: Option<String>,
    pub primary_keys: Vec<String>,

    pub pending_edits: HashMap<(usize, usize), String>,
    pub pending_deletes: HashSet<usize>,
    pub pending_inserts: Vec<Vec<String>>,

    pub limit: usize,
    pub offset: usize,
    pub total_rows: Option<usize>,
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
            is_loading: false,
            editing_state: None,
        }
    }

    pub fn is_editable(&self) -> bool {
        self.source_table.is_some() && !self.primary_keys.is_empty()
    }

    pub fn has_pending_changes(&self) -> bool {
        !self.pending_edits.is_empty()
            || !self.pending_deletes.is_empty()
            || !self.pending_inserts.is_empty()
    }
}
