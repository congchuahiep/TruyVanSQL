#[derive(Debug, Clone)]
pub struct ColumnData {
    pub column_name: String,
    pub value: String,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct RowUpdate {
    pub pk_conditions: Vec<ColumnData>,
    pub changes: Vec<ColumnData>,
}

#[derive(Debug, Clone)]
pub struct RowDelete {
    pub pk_conditions: Vec<ColumnData>,
}

#[derive(Debug, Clone)]
pub struct DataChangeset {
    pub table_name: String,
    pub updates: Vec<RowUpdate>,
    pub deletes: Vec<RowDelete>,
}
