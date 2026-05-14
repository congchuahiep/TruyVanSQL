/// Thông tin chi tiết của một column trong table.
///
/// Chứa mọi metadata cần thiết cho UI: tên, kiểu, nullable, default, PK.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Tên column
    pub name: String,
    /// Kiểu dữ liệu (ví dụ: "INTEGER", "TEXT", "VARCHAR(255)")
    pub data_type: String,
    /// `true` nếu column cho phép NULL (không có NOT NULL constraint)
    pub nullable: bool,
    /// Giá trị default (None nếu không có DEFAULT)
    pub default_value: Option<String>,
    /// `true` nếu column thuộc primary key
    pub is_primary_key: bool,
}