use super::{DatabaseKind, FileDbConfig, NetworkDbConfig, NetworkParams};

/// Cấu hình kết nối database — enum chứa config riêng cho từng loại.
///
/// Sử dụng các convenience constructor (`sqlite`, `postgres`) để tạo nhanh.
#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Sqlite(FileDbConfig),
    Network(NetworkDbConfig),
}

impl DatabaseConfig {
    /// Convenience constructor cho SQLite.
    ///
    /// # Ví dụ
    /// ```ignore
    /// let config = DatabaseConfig::sqlite("sqlite:./mydata.db");
    /// let config = DatabaseConfig::sqlite("sqlite::memory:");
    /// ```
    pub fn sqlite(url: impl Into<String>) -> Self {
        Self::Sqlite(FileDbConfig {
            url: url.into(),
            max_connections: 5,
        })
    }

    /// Convenience constructor cho các database network (Postgres, MySQL, etc).
    ///
    /// # Ví dụ
    /// ```ignore
    /// let config = DatabaseConfig::network(DatabaseKind::Postgres, "localhost", 5432, "user", "password", "mydb");
    /// ```
    pub fn network(
        kind: DatabaseKind,
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Self {
        Self::Network(NetworkDbConfig {
            kind,
            network: NetworkParams {
                host: host.to_string(),
                port,
                user: user.to_string(),
                password: password.to_string(),
                database: database.to_string(),
            },
            max_connections: 5,
        })
    }

    /// Trả về loại database.
    pub fn kind(&self) -> DatabaseKind {
        match self {
            Self::Sqlite(_) => DatabaseKind::Sqlite,
            Self::Network(_) => DatabaseKind::Postgres,
        }
    }

    /// Trả về số lượng connection tối đa.
    pub fn max_connections(&self) -> u32 {
        match self {
            Self::Sqlite(c) => c.max_connections,
            Self::Network(c) => c.max_connections,
        }
    }

    /// Trả về connection URL phù hợp cho sqlx.
    pub fn connection_url(&self) -> String {
        match self {
            Self::Sqlite(c) => c.url.clone(),
            Self::Network(c) => c.connection_url(),
        }
    }
}
