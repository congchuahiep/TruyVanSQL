/// Cấu hình kết nối tệp Database, thông thường là SQLite
#[derive(Debug, Clone)]
pub struct FileDbConfig {
    /// Connection URL (ví dụ: "sqlite:./data.db", "sqlite::memory:")
    pub url: String,
    /// Số lượng connection tối đa trong pool
    pub max_connections: u32,
    /// Thời gian timeout khi acquire connection từ pool (giây).
    /// Áp dụng cho cả lần kết nối đầu tiên. Mặc định là 10 giây
    pub acquire_timeout_secs: u64,
}

impl Default for FileDbConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
            acquire_timeout_secs: 10,
        }
    }
}
