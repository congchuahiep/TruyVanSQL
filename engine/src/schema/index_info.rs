/// Thông tin index.
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Tên index
    pub name: String,
    /// Column(s) trong index, theo thứ tự
    pub columns: Vec<String>,
    /// `true` nếu là UNIQUE index
    pub is_unique: bool,
}