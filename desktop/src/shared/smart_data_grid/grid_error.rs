use gpui::SharedString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StageError {
    #[error("Dữ liệu không hợp lệ: {0}")]
    InvalidData(String),

    #[error("Không có phiên chỉnh sửa nào đang hoạt động")]
    NoActiveEdit,
}

// GridError: UI state enum — quyết định cách hiển thị lỗi trên grid
#[derive(Clone, Debug)]
pub enum GridError {
    /// Lỗi nghiêm trọng (không load được bảng)
    Fatal(SharedString),
    /// Lỗi commit changes
    Commit(SharedString),
    /// Không bị gì cả
    None,
}

impl GridError {
    /// Kiểm tra lỗi hiện tại có phải lỗi nghiêm trọng không (không thể load được bảng)
    pub fn is_fatal(&self) -> bool {
        matches!(self, GridError::Fatal(_))
    }

    /// Lấy message từ GridError. Trả về chuỗi rỗng nếu None.
    pub fn message(&self) -> SharedString {
        match self {
            GridError::Fatal(msg) | GridError::Commit(msg) => msg.clone(),
            GridError::None => SharedString::new(""),
        }
    }
}
