use super::TableKind;

#[derive(Debug, Clone)]
pub struct TableBrief {
    pub name: String,
    pub kind: TableKind,
    pub schema_name: Option<String>,
}