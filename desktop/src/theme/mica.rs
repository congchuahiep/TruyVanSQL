#[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
pub fn sync_mica_dark_mode(_window: &gpui::Window, _is_dark: bool) {}
