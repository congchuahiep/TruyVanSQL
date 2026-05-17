/// Khóa chính của table.
///
/// Composite primary key chứa nhiều columns.
#[derive(Debug, Clone)]
pub struct PrimaryKey {
    /// Danh sách column(s) trong PK, theo thứ tự
    pub columns: Vec<String>,
}