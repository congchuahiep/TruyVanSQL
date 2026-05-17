use assets::AppIcon;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::DropdownMenu;
use gpui_component::menu::PopupMenu;
use gpui_component::{ActiveTheme, InteractiveElementExt, StyledExt, ThemeRegistry, h_flex};

use crate::action::app::Quit;
use crate::action::connection::ConnectDatabase;
use crate::action::query::{ExecuteQuery, NewQuery};
use crate::action::theme::SwitchTheme;
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::theme;

pub enum TitlebarKind {
    Main,
    Dialog { title: SharedString },
}

pub struct Titlebar {
    should_move: bool,
    kind: TitlebarKind,
}

impl Titlebar {
    pub fn main(_cx: &mut Context<Self>) -> Self {
        Self {
            should_move: false,
            kind: TitlebarKind::Main,
        }
    }

    pub fn dialog(title: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self {
        Self {
            should_move: false,
            kind: TitlebarKind::Dialog {
                title: title.into(),
            },
        }
    }

    fn on_toggle_dark_light(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        theme::toggle_dark_light(cx);
    }

    fn render_theme_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = cx.theme().is_dark();
        let icon = if is_dark { AppIcon::Sun } else { AppIcon::Moon };
        Button::new("btn-theme-toggle")
            .ghost()
            .size_6()
            .cursor_pointer()
            .icon(icon)
            .on_click(cx.listener(|this, _, window, cx| {
                this.on_toggle_dark_light(window, cx);
            }))
    }

    fn render_main_drag_area(&self) -> impl IntoElement {
        div()
            .id("window-drag")
            .window_control_area(WindowControlArea::Drag)
            .flex_1()
            .h_full()
            .flex()
            .items_center()
            .gap_1()
            .pl_0p5()
            .child(
                Button::new("hamburger-menu")
                    .ghost()
                    .cursor_pointer()
                    .icon(AppIcon::Menu)
                    .dropdown_menu(move |menu: PopupMenu, window, cx| {
                        menu.menu("New Database", Box::new(NewDatabase))
                            .menu("Open File...", Box::new(OpenFile))
                            .menu("In-Memory Database", Box::new(UseInMemory))
                            .separator()
                            .menu("Kết nối...", Box::new(ConnectDatabase))
                            .separator()
                            .submenu("Theme", window, cx, |menu: PopupMenu, _window, cx| {
                                let themes = ThemeRegistry::global(cx).sorted_themes();
                                let current_name = cx.theme().theme_name();
                                let mut menu = menu;
                                for t in &themes {
                                    let checked = current_name == &t.name;
                                    menu = menu.menu_with_check(
                                        &t.name,
                                        checked,
                                        Box::new(SwitchTheme(t.name.clone())),
                                    );
                                }
                                menu
                            })
                            .separator()
                            .menu("New Query", Box::new(NewQuery))
                            .menu("Run Query", Box::new(ExecuteQuery))
                            .separator()
                            .menu("Quit", Box::new(Quit))
                    }),
            )
    }

    fn render_main_window_controls(&self, window: &mut Window) -> impl IntoElement {
        if cfg!(target_os = "macos") || cfg!(target_family = "wasm") {
            return h_flex().id("window-controls");
        }

        let is_maximized = window.is_maximized();
        let restore_icon = if is_maximized {
            AppIcon::WindowRestore
        } else {
            AppIcon::WindowMaximize
        };

        h_flex()
            .id("window-controls")
            .child(
                Button::new("btn-minimize")
                    .ghost()
                    .size_10()
                    .rounded_none()
                    .cursor_pointer()
                    .icon(AppIcon::WindowMinimize)
                    .window_control_area(WindowControlArea::Min)
                    .on_click(|_, window, _| {
                        window.minimize_window();
                    }),
            )
            .child(
                Button::new("btn-maximize")
                    .ghost()
                    .size_10()
                    .rounded_none()
                    .cursor_pointer()
                    .icon(restore_icon)
                    .window_control_area(WindowControlArea::Max)
                    .on_click(|_, window, _| {
                        window.zoom_window();
                    }),
            )
            .child(
                Button::new("btn-close")
                    .ghost()
                    .size_10()
                    .rounded_none()
                    .cursor_pointer()
                    .icon(AppIcon::WindowClose)
                    .window_control_area(WindowControlArea::Close)
                    .on_click(|_, window, cx| {
                        window.remove_window();
                        cx.quit();
                    }),
            )
    }

    /// Drag area: chỉ để bắt drag, KHÔNG chứa tiêu đề.
    /// Chiếm flex_1 giữa các controls, không full-width.
    fn render_dialog_drag_area(&self) -> impl IntoElement {
        h_flex()
            .id("titlebar-drag")
            .window_control_area(WindowControlArea::Drag)
            .flex_1()
            .h_full()
    }

    /// Tiêu đề: absolute, nằm giữa TOÀN BỘ titlebar.
    fn render_dialog_title(&self) -> impl IntoElement {
        let title = match &self.kind {
            TitlebarKind::Dialog { title } => title.clone(),
            _ => SharedString::default(),
        };
        h_flex()
            .id("dialog-title-overlay")
            .absolute()
            .left_0()
            .right_0()
            .h_full()
            .items_center()
            .justify_center()
            .child(div().text_xs().font_semibold().child(title.clone()))
    }

    /// Controls: flex item bình thường, không absolute.
    fn render_dialog_controls(&self) -> impl IntoElement {
        if cfg!(target_os = "macos") || cfg!(target_family = "wasm") {
            return h_flex().id("dialog-controls");
        }
        h_flex().id("dialog-controls").flex_shrink_0().child(
            Button::new("btn-dialog-close")
                .ghost()
                .size_10()
                .rounded_none()
                .cursor_pointer()
                .icon(AppIcon::WindowClose)
                .window_control_area(WindowControlArea::Close)
                .on_click(|_, window, _| {
                    window.remove_window();
                }),
        )
    }
}

impl Render for Titlebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let el = h_flex()
            .id("titlebar")
            .key_context("titlebar")
            .w_full()
            .h(px(40.))
            .flex_shrink_0()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.should_move = true;
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.should_move = false;
                }),
            )
            .on_mouse_down_out(cx.listener(|this, _, _, _| {
                this.should_move = false;
            }))
            .on_mouse_move(cx.listener(move |this, _, window, _| {
                if this.should_move {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            .on_double_click(cx.listener(|_, _, window, _| {
                window.zoom_window();
            }));

        match &self.kind {
            TitlebarKind::Main => el
                .child(self.render_main_drag_area())
                .child(div().id("theme-toggle").child(self.render_theme_toggle(cx)))
                .child(self.render_main_window_controls(window))
                .into_any_element(),
            TitlebarKind::Dialog { .. } => el
                .relative()
                .child(self.render_dialog_drag_area())
                .child(self.render_dialog_controls())
                .child(self.render_dialog_title())
                .into_any_element(),
        }
    }
}
