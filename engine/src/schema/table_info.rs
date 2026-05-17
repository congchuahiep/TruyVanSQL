use super::{ColumnInfo, ForeignKeyInfo, IndexInfo, PrimaryKey};

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