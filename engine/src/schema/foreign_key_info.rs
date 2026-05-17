/// Thông tin foreign key.
///
/// Mô tả mối quan hệ giữa table hiện tại và table khác.
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    /// Tên FK constraint (None nếu database không hỗ trợ đặt tên FK, ví dụ: SQLite)
    pub name: Option<String>,
    /// Column(s) trong table hiện tại
    pub columns: Vec<String>,
    /// Tên table được tham chiếu
    pub references_table: String,
    /// Column(s) trong table được tham chiếu
    pub references_columns: Vec<String>,
    /// Hành động khi xóa (CASCADE, SET NULL, SET DEFAULT, RESTRICT, NO ACTION, None)
    pub on_delete: Option<String>,
    /// Hành động khi cập nhật
    pub on_update: Option<String>,
}