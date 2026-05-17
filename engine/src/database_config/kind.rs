/// Loại database được hỗ trợ.
///
/// Mỗi variant tương ứng với một driver implementation.
/// Khi thêm database mới, chỉ cần thêm variant tại đây và implement
/// `DatabaseDriver` trait trong `driver` module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
}

/// Phân loại kết nối database.
///
/// Việc phân loại này giúp xác định cách thức kết nối:
/// - File-based (SQLite)
/// - Network-based (PostgreSQL, MySQL,... Hầu hết mọi loại database khác)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionCategory {
    FileBased,
    NetworkBased,
}

impl DatabaseKind {
    /// Phân loại kết nối: File-based hay Network-based.
    pub fn category(&self) -> ConnectionCategory {
        match self {
            Self::Sqlite => ConnectionCategory::FileBased,
            Self::Postgres => ConnectionCategory::NetworkBased,
        }
    }
    /// Port mặc định cho từng loại database.
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Sqlite => 0,
            Self::Postgres => 5432,
        }
    }
    /// User mặc định.
    pub fn default_user(&self) -> &'static str {
        match self {
            Self::Sqlite => "",
            Self::Postgres => "postgres",
        }
    }
    /// Database mặc định.
    pub fn default_database(&self) -> &'static str {
        match self {
            Self::Sqlite => "",
            Self::Postgres => "postgres",
        }
    }
    /// URL scheme cho connection string.
    pub fn url_scheme(&self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgresql",
        }
    }
    /// Tên hiển thị cho UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "PostgreSQL",
        }
    }
}
