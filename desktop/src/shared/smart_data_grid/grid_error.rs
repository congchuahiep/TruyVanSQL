use gpui::SharedString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StageError {
    #[error("Dữ liệu không hợp lệ: {0}")]
    InvalidData(String),

    #[error("Không có phiên chỉnh sửa nào đang hoạt động")]
    NoActiveEdit,
}

#[derive(Clone, Debug)]
pub enum GridError {
    /// Lỗi nghiêm trọng (không load được bảng), thường sử dụng khi bảng không thể load được lúc
    /// khởi tạo
    Fatal(SharedString),
    /// Lỗi xảy ra khi refresh không thành công
    Refresh(SharedString),
    /// Lỗi commit changes
    Commit(SharedString),
}

impl GridError {
    /// Kiểm tra lỗi hiện tại có phải lỗi nghiêm trọng không (không thể load được bảng)
    pub fn is_fatal(&self) -> bool {
        matches!(self, GridError::Fatal(_))
    }

    /// Lấy message từ GridError. Trả về chuỗi rỗng nếu None.
    pub fn message(&self) -> SharedString {
        match self {
            GridError::Fatal(msg) | GridError::Refresh(msg) | GridError::Commit(msg) => msg.clone(),
        }
    }
}
