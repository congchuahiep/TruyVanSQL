use engine::{Column, Row};

#[derive(Debug, Clone)]
pub enum OutputContent {
    Empty,
    Execution { text: String },
    Query { columns: Vec<Column>, rows: Vec<Row> },
    Error(String),
}
