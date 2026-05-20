#[derive(Debug, Clone)]
pub struct DatabaseBrief {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SchemaBrief {
    pub name: String,
    pub kind: SchemaKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaKind {
    User,
    System,
}