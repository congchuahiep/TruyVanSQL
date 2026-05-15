#[derive(Debug, Clone)]
pub enum OutputContent {
    Empty,
    Execution { text: String },
    Query { columns: Vec<engine::Column>, rows: Vec<engine::Row> },
    Error(String),
}