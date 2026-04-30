/// Loại database được hỗ trợ.
///
/// Mỗi variant tương ứng với một driver implementation.
/// Khi thêm database mới, chỉ cần thêm variant tại đây và implement
/// `DatabaseDriver` trait trong `driver` module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseKind {
    Sqlite,
    // TODO: Tương lai thêm Postgres, MySQL, MSSQL...
}

/// Cấu hình kết nối database.
///
/// Chứa mọi thông tin cần thiết để tạo một database driver.
/// Sử dụng các convenience constructor (`sqlite`, `postgres`, ...) để tạo nhanh.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Loại database
    pub kind: DatabaseKind,
    /// Connection URL (ví dụ: "sqlite:./data.db", "postgres://user:pass@localhost/db")
    pub url: String,
    /// Số lượng connection tối đa trong pool
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Tạo config mới với đầy đủ thông tin.
    pub fn new(kind: DatabaseKind, url: impl Into<String>, max_connections: u32) -> Self {
        Self {
            kind,
            url: url.into(),
            max_connections,
        }
    }

    /// Convenience constructor cho SQLite.
    ///
    /// # Ví dụ
    /// ```ignore
    /// let config = DatabaseConfig::sqlite("sqlite:./mydata.db");
    /// let config = DatabaseConfig::sqlite("sqlite::memory:");
    /// ```
    pub fn sqlite(url: impl Into<String>) -> Self {
        Self {
            kind: DatabaseKind::Sqlite,
            url: url.into(),
            max_connections: 5,
        }
    }
}
