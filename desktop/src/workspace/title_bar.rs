use assets::AppIcon;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::DropdownMenu;
use gpui_component::menu::PopupMenu;
use gpui_component::{ActiveTheme, InteractiveElementExt, ThemeRegistry, h_flex};

use crate::action::app::Quit;
use crate::action::query::{ExecuteQuery, NewQuery};
use crate::action::theme::SwitchTheme;
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::theme;

pub struct Titlebar {
    should_move: bool,
}

impl Titlebar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { should_move: false }
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

    fn render_window_controls(
        &self,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
            .gap_0p5()
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
}

impl Render for Titlebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
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
            }))
            .child(
                div()
                    .id("titlebar-drag")
                    .window_control_area(WindowControlArea::Drag)
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .gap_1()
                    .pl_1()
                    .child(
                        Button::new("hamburger-menu")
                            .ghost()
                            .size_8()
                            .cursor_pointer()
                            .icon(AppIcon::Menu)
                            .dropdown_menu(move |menu: PopupMenu, window, cx| {
                                menu.menu("New Database", Box::new(NewDatabase))
                                    .menu("Open File...", Box::new(OpenFile))
                                    .menu("In-Memory Database", Box::new(UseInMemory))
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
                    ),
            )
            .child(div().id("theme-toggle").child(self.render_theme_toggle(cx)))
            .child(self.render_window_controls(_window, cx))
    }
}
