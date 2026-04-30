use thiserror::Error;

/// Lỗi chính của Engine.
///
/// Mọi operation trong engine đều trả về `Result<T, EngineError>`.
/// Các variant phân loại lỗi theo tầng xử lý, giúp caller xử lý phù hợp.
#[derive(Error, Debug)]
pub enum EngineError {
    /// Lỗi khi thiết lập hoặc duy trì kết nối database.
    ///
    /// Bao gồm: connection refused, timeout, authentication failed, etc.
    #[error("Lỗi kết nối: {0}")]
    Connection(String),

    /// Lỗi khi thực thi SQL query.
    ///
    /// Bao gồm: syntax error, constraint violation, permission denied, etc.
    #[error("Lỗi thực thi truy vấn: {0}")]
    QueryExecution(String),

    /// Lỗi khi truy vấn schema metadata.
    ///
    /// Bao gồm: table không tồn tại, column không hợp lệ, PRAGMA error, etc.
    #[error("Lỗi schema: {0}")]
    Schema(String),

    /// Database type không được hỗ trợ.
    ///
    /// Xảy ra khi `DatabaseKind` chưa có driver implementation tương ứng.
    #[error("Database không được hỗ trợ: {0}")]
    UnsupportedDatabase(String),
}
