/// Đồng bộ dark/light mode cho Mica backdrop trên Windows.
///
/// Khi feature `mica` được bật, gọi DWM API để set immersive dark mode.
/// Khi feature tắt, hàm là no-op (không làm gì).
#[cfg(all(target_os = "windows", feature = "mica"))]
pub fn sync_mica_dark_mode(window: &gpui::Window, is_dark: bool) {
    use raw_window_handle::HasWindowHandle;
    use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};

    let handle = match HasWindowHandle::window_handle(window) {
        Ok(h) => h,
        Err(_) => return,
    };
    let raw = handle.as_raw();
    if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = raw {
        let hwnd = windows::Win32::Foundation::HWND(win32_handle.hwnd.get() as *mut _);
        let dark_mode: i32 = if is_dark { 1 } else { 0 };
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_mode as *const _ as *const _,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }
}
