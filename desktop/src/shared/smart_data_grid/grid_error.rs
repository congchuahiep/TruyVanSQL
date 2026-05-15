use thiserror::Error;

#[derive(Error, Debug)]
pub enum StageError {
    #[error("Dữ liệu không hợp lệ: {0}")]
    InvalidData(String),

    #[error("Không có phiên chỉnh sửa nào đang hoạt động")]
    NoActiveEdit,
}