use assets::AppIcon;
use gpui::*;
use std::any::Any;

pub struct TabInfo {
    /// Tiêu đề hiển thị trên thanh Tab (ví dụ: tên database, tên file)
    pub title: SharedString,

    /// Kiểm tra tab có dữ liệu chưa lưu hay không (hiển thị dấu chấm tròn)
    pub is_dirty: bool,

    /// Icon hiển thị trên thanh Tab (ví dụ: biểu tượng database, biểu tượng file)
    pub icon: AppIcon,
}

/// Trait cốt lõi định nghĩa một Tab bất kỳ trong hệ thống.
/// Mọi tính năng muốn hiển thị trên thanh Tab đều phải implement Trait này.
pub trait TabItem: Render {
    /// Thông tin của tab hiện tại như tiêu đề tab, trạng thái dirty (hiển thị dấu chấm tròn),...
    fn tab_info(&self, cx: &App) -> TabInfo;

    /// Trả về tham chiếu Any để TabManager có thể quản lý đa hình (Type Erasure)
    fn as_any(&self) -> &dyn Any;
}