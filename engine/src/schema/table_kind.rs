/// Loại đối tượng trong database.
///
/// Phân biệt table, view, và system objects (bảng hệ thống của database).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableKind {
    /// Bảng thường do người dùng tạo
    Table,
    /// View
    View,
    /// Bảng hệ thống (ví dụ: `sqlite_sequence` trong SQLite)
    System,
}