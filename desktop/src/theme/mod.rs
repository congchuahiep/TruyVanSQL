//! Module quản lý theme cho ứng dụng.
//!
//! Cung cấp các hàm để:
//! - Khởi tạo theme từ folder `themes/`
//! - Chuyển đổi theme theo tên
//! - Toggle giữa Light và Dark mode
pub mod mica;

use gpui::{App, SharedString, WindowBackgroundAppearance};
use gpui_component::{ActiveTheme as _, Theme, ThemeMode, ThemeRegistry};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::action::theme::{SwitchTheme, SwitchThemeMode};

const DEFAULT_THEME_NAME: &str = "TruyVanSQL Light";
const THEMES_DIR: &str = "themes";
static THEME_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Khởi tạo hệ thống theme.
///
/// Gọi hàm này một lần trong `main()` sau `gpui_component::init(cx)`.
/// Hàm sẽ:
/// 1. Load tất cả file JSON theme từ folder `themes/`
/// 2. Watch folder để tự động reload khi file thay đổi
/// 3. Áp dụng theme mặc định
/// 4. Đăng ký action handlers cho SwitchTheme và SwitchThemeMode
/// 5. Đồng bộ Mica backdrop dark/light mode với app theme
pub fn init(cx: &mut App) {
    // Tránh khởi tạo nhiều lần
    if THEME_INITIALIZED.get().is_some() {
        return;
    }
    let default_theme = SharedString::from(DEFAULT_THEME_NAME);
    // Load và watch tất cả file JSON trong folder themes/
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from(THEMES_DIR), cx, move |cx| {
        // Khi load xong (hoặc file thay đổi), áp dụng theme mặc định
        apply_theme_by_name(&default_theme, cx);
    }) {
        // Folder themes/ không tìm thấy — dùng theme mặc định của gpui-component
        tracing::warn!(
            "Không thể watch folder themes/: {}. Dùng theme mặc định.",
            err
        );
    }

    // Đăng ký action handler cho SwitchTheme
    cx.on_action(|action: &SwitchTheme, cx| {
        change_theme(action.0.clone(), cx);
        cx.refresh_windows();
    });

    // Đăng ký action handler cho SwitchThemeMode
    cx.on_action(|action: &SwitchThemeMode, cx| {
        Theme::change(action.0, None, cx);
        cx.refresh_windows();
    });

    // Đồng bộ Mica backdrop dark/light mode khi theme thay đổi
    cx.observe_global::<Theme>(|cx| {
        let is_dark = cx.theme().is_dark();
        for window in cx.windows().iter_mut() {
            window
                .update(cx, |_, window, _| {
                    window.set_background_appearance(WindowBackgroundAppearance::MicaBackdrop);
                    mica::sync_mica_dark_mode(window, is_dark);
                })
                .ok();
        }
    })
    .detach();

    THEME_INITIALIZED.set(()).ok();
}

/// Chuyển sang theme theo tên.
///
/// Tìm theme trong ThemeRegistry và áp dụng.
/// Nếu không tìm thấy, giữ nguyên theme hiện tại.
///
/// # Ví dụ
///
/// ```ignore
/// theme::change_theme(SharedString::from("Default Dark"), cx);
/// ```
pub fn change_theme(name: SharedString, cx: &mut App) {
    apply_theme_by_name(&name, cx);
}

/// Toggle giữa Light mode và Dark mode.
///
/// Nếu đang Light → chuyển sang Dark, và ngược lại.
/// Giữ nguyên tên theme, chỉ đổi mode.
///
/// # Ví dụ
///
/// ```ignore
/// // Gọi từ event handler:
/// theme::toggle_dark_light(cx);
/// ```
pub fn toggle_dark_light(cx: &mut App) {
    let new_mode = if cx.theme().is_dark() {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };
    Theme::change(new_mode, None, cx);
}

/// Áp dụng theme config theo tên.
///
/// Tìm theme trong registry, nếu tìm thấy thì áp dụng
/// cả light_theme và dark_theme vào global Theme.
fn apply_theme_by_name(name: &SharedString, cx: &mut App) {
    let registry = ThemeRegistry::global(cx);
    let Some(theme_config) = registry.themes().get(name).cloned() else {
        tracing::warn!("Theme '{}' không tìm thấy trong registry.", name);
        return;
    };
    let mode = theme_config.mode;
    let theme = Theme::global_mut(cx);
    // Áp dụng config vào light_theme hoặc dark_theme tương ứng
    if mode.is_dark() {
        theme.dark_theme = theme_config;
    } else {
        theme.light_theme = theme_config;
    }
    // Kích hoạt mode tương ứng
    Theme::change(mode, None, cx);
}
