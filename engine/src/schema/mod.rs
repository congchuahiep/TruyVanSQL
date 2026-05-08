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

/// Thông tin tóm tắt của một table/view — dùng cho sidebar listing.
///
/// Nhẹ, chỉ chứa name và kind. Dùng [`SqlClient::list_tables`] để lấy.
/// Khi cần chi tiết (columns, PK, FK), dùng [`SqlClient::get_table_info`].
#[derive(Debug, Clone)]
pub struct TableBrief {
    /// Tên table/view
    pub name: String,
    /// Loại đối tượng
    pub kind: TableKind,
}

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

/// Khóa chính của table.
///
/// Composite primary key chứa nhiều columns.
#[derive(Debug, Clone)]
pub struct PrimaryKey {
    /// Danh sách column(s) trong PK, theo thứ tự
    pub columns: Vec<String>,
}

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

/// Thông tin đầy đủ của một table — trả về khi click vào table.
///
/// Chứa columns, primary key, foreign keys, và indexes.
/// Dùng [`SqlClient::get_table_info`] để lấy.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Tên table
    pub name: String,
    /// Danh sách columns
    pub columns: Vec<ColumnInfo>,
    /// Primary key (có thể composite)
    pub primary_key: PrimaryKey,
    /// Danh sách foreign keys
    pub foreign_keys: Vec<ForeignKeyInfo>,
    /// Danh sách indexes
    pub indexes: Vec<IndexInfo>,
}

pub mod modification;
pub use modification::{ColumnData, DataChangeset, RowDelete, RowUpdate};
