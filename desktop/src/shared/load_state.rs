/// Trạng thái của một async fetch.
///
/// Flow: Idle → Loading → Loaded(T)
#[derive(Debug, Clone)]
pub enum LoadState<T> {
    /// Chưa trigger fetch
    Idle,
    /// Đang fetch dữ liệu
    Loading,
    /// Dữ liệu đã được fetch và lưu trữ
    Loaded(T),
    /// Fetch thất bại
    Error(LoadError),
}

impl<T> LoadState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadState::Loading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadState::Loaded(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, LoadState::Error(_))
    }

    pub fn as_loaded(&self) -> Option<&T> {
        match self {
            LoadState::Loaded(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_loaded_mut(&mut self) -> Option<&mut T> {
        match self {
            LoadState::Loaded(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_error(&self) -> Option<&LoadError> {
        match self {
            LoadState::Error(err) => Some(err),
            _ => None,
        }
    }
}

/// Lỗi khi fetch dữ liệu async
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    /// Không kết nối được đến database
    Connection(String),
    /// Timeout
    Timeout(String),
    /// Lỗi query/SQL
    Query(String),
    /// Không tìm thấy resource
    NotFound(String),
    /// Lỗi không xác định
    Unknown(String),
}

impl LoadError {
    /// Hiển thị message cho user
    pub fn message(&self) -> &str {
        match self {
            LoadError::Connection(msg)
            | LoadError::Timeout(msg)
            | LoadError::Query(msg)
            | LoadError::NotFound(msg)
            | LoadError::Unknown(msg) => msg,
        }
    }
}
