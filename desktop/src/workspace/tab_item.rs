use gpui::*;
use std::any::Any;

/// Trait cốt lõi định nghĩa một Tab bất kỳ trong hệ thống.
/// Mọi tính năng muốn hiển thị trên thanh Tab đều phải implement Trait này.
pub trait TabItem: Render {
    /// Tiêu đề hiển thị trên thanh Tab (ví dụ: tên database, tên file)
    fn tab_title(&self, cx: &App) -> SharedString;

    /// Kiểm tra tab có dữ liệu chưa lưu hay không (hiển thị dấu chấm tròn thay vì dấu X)
    fn is_dirty(&self, _cx: &App) -> bool {
        false
    }

    /// Trả về tham chiếu Any để TabManager có thể quản lý đa hình (Type Erasure)
    fn as_any(&self) -> &dyn Any;
}
