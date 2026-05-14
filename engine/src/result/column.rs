/// Metadata của một cột trong kết quả query.
#[derive(Debug, Clone)]
pub struct Column {
    /// Tên cột
    pub name: String,
    /// Kiểu dữ liệu declared trong schema (có thể None nếu query tạo ra computed column)
    pub declared_type: Option<String>,
}