/// Cấu hình kết nối tệp Database, thông thường là SQLite
#[derive(Debug, Clone)]
pub struct FileDbConfig {
    /// Connection URL (ví dụ: "sqlite:./data.db", "sqlite::memory:")
    pub url: String,
    /// Số lượng connection tối đa trong pool
    pub max_connections: u32,
}

impl Default for FileDbConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
        }
    }
}
