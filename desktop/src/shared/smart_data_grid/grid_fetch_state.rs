use super::GridError;

#[derive(Debug, Clone)]
pub enum GridFetchState {
    Idle,
    Loading,
    Loaded,
    Error(GridError),
}

impl GridFetchState {
    pub fn is_loading(&self) -> bool {
        matches!(self, GridFetchState::Loading)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, GridFetchState::Idle)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, GridFetchState::Loaded)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, GridFetchState::Error(_))
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self, GridFetchState::Error(GridError::Fatal(_)))
    }

    /// Kiểm tra đã có dữ liệu fetch hay chưa (đã từng hoặc đang có), dùng để xác định có thể hiển
    /// thị data từ GridState không?
    ///
    /// - `Loaded`: data vừa fetch xong, hiển thị bình thường
    /// - `Error(Refresh)`: data cũ vẫn còn, hiển thị kèm thông báo lỗi
    /// - `Error(Commit)`: data hợp lệ, chỉ commit thất bại
    /// - Các state khác: không có data tin cậy để hiển thị
    pub fn has_displayable_data(&self) -> bool {
        matches!(
            self,
            Self::Loaded | Self::Error(GridError::Refresh(_)) | Self::Error(GridError::Commit(_))
        )
    }

    /// Trả về lỗi nếu có, None nếu không có
    pub fn as_error(&self) -> Option<&GridError> {
        match self {
            GridFetchState::Error(e) => Some(e),
            _ => None,
        }
    }
}
