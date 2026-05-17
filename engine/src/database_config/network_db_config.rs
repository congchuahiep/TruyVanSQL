use super::DatabaseKind;

/// Cấu hình kết nối các database network-based.
///
/// Dùng chung cho PostgreSQL, MySQL, và bất kỳ DB nào
/// có form kết nối host:port + auth.
#[derive(Debug, Clone)]
pub struct NetworkDbConfig {
    pub kind: DatabaseKind,
    pub network: NetworkParams,
    pub max_connections: u32,
    /// Thời gian timeout khi acquire connection từ pool (giây).
    /// Áp dụng cho cả lần kết nối đầu tiên. Mặc định là 10 giây.
    /// - `None` = không giới hạn (có thể treo đến 1-2 phút với host không tồn tại).
    pub acquire_timeout_secs: u64,
}

impl NetworkDbConfig {
    /// Sinh connection URL từ tham số mạng.
    pub fn connection_url(&self) -> String {
        let n = &self.network;
        match self.kind {
            DatabaseKind::Postgres => {
                if n.password.is_empty() {
                    format!(
                        "postgresql://{}@{}:{}/{}",
                        n.user, n.host, n.port, n.database
                    )
                } else {
                    format!(
                        "postgresql://{}:{}@{}:{}/{}",
                        n.user, n.password, n.host, n.port, n.database
                    )
                }
            }
            _ => unreachable!("NetworkDbConfig chỉ dùng cho network-based databases"),
        }
    }
}

/// Tham số kết nối mạng chung cho các database client-server.
///
/// Được dùng chung bởi PostgreSQL, MySQL, và bất kỳ database nào
/// sử dụng mô hình host:port + user + password + database.
#[derive(Debug, Clone)]
pub struct NetworkParams {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl Default for NetworkParams {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 0, // Sẽ được ghi đè bởi default_port() của DatabaseKind
            user: String::new(),
            password: String::new(),
            database: String::new(),
        }
    }
}
