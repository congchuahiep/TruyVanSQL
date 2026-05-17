use super::Value;

/// Một dòng dữ liệu trong kết quả query.
///
/// Mỗi `Row` chứa danh sách `Option<Value>`, thứ tự tương ứng với thứ tự cột.
/// `None` đại diện cho giá trị NULL.
#[derive(Debug, Clone)]
pub struct Row {
    pub values: Vec<Option<Value>>,
}