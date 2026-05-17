use super::TableKind;

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