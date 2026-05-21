use engine::EngineError;

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
    Error(EngineError),
}

impl<T> LoadState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadState::Loading)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadState::Loaded(_))
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, LoadState::Idle)
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

    pub fn as_error(&self) -> Option<&EngineError> {
        match self {
            LoadState::Error(err) => Some(err),
            _ => None,
        }
    }
}
