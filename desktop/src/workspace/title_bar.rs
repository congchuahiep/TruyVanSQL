use assets::AppIcon;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::AppMenuBar;
use gpui_component::{ActiveTheme, GlobalState, Theme, ThemeRegistry, h_flex};

use crate::action::app::Quit;
use crate::action::query::{ExecuteQuery, NewQuery};
use crate::action::theme::SwitchTheme;
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::theme;

pub struct Titlebar {
    app_menu: Entity<AppMenuBar>,
}

impl Titlebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let menus = build_menus(cx);
        GlobalState::global_mut(cx).set_app_menus(menus.into_iter().map(|m| m.owned()).collect());
        let app_menu = AppMenuBar::new(cx);

        // Theo dõi sự thay đổi theme để cập nhật menu
        cx.observe_global::<Theme>(|this, cx| {
            let menus = build_menus(cx);
            GlobalState::global_mut(cx)
                .set_app_menus(menus.into_iter().map(|m| m.owned()).collect());
            this.app_menu.update(cx, |menu_bar, cx| {
                menu_bar.reload(cx);
            });
        })
        .detach();

        Self { app_menu }
    }

    /// Toggle giữa Light và Dark mode
    fn on_toggle_dark_light(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        theme::toggle_dark_light(cx);
    }

    fn render_theme_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = cx.theme().is_dark();
        let icon = if is_dark {
            AppIcon::Sun // Dark mode → hiện nút chuyển sang Light
        } else {
            AppIcon::Moon // Light mode → hiện nút chuyển sang Dark
        };
        Button::new("btn-theme-toggle")
            .ghost()
            .size_6()
            .cursor_pointer()
            .icon(icon)
            .on_click(cx.listener(|this, _, window, cx| {
                this.on_toggle_dark_light(window, cx);
            }))
    }
}

impl Render for Titlebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id("titlebar")
            .key_context("titlebar")
            .w_full()
            .h(px(32.))
            .px_1()
            .flex_shrink_0()
            .child(
                div()
                    .id("menu-bar")
                    .key_context("menu-bar")
                    .flex_1()
                    .h_full()
                    .child(self.app_menu.clone()),
            )
            .child(div().id("theme-toggle").child(self.render_theme_toggle(cx)))
    }
}

fn build_menus(cx: &App) -> Vec<Menu> {
    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Database", NewDatabase),
                MenuItem::action("Open File...", OpenFile),
                MenuItem::action("In-Memory Database", UseInMemory),
                MenuItem::separator(),
                theme_menu(cx),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "Query".into(),
            items: vec![
                MenuItem::action("New Query", NewQuery),
                MenuItem::separator(),
                MenuItem::action("Run Query", ExecuteQuery),
            ],
            disabled: false,
        },
        Menu {
            name: "Window".into(),
            items: vec![],
            disabled: false,
        },
    ]
}

/// Tạo submenu Theme với danh sách tất cả theme có sẵn.
///
/// Mỗi item có checkmark nếu đang active.
fn theme_menu(cx: &App) -> MenuItem {
    let themes = ThemeRegistry::global(cx).sorted_themes();
    let current_name = cx.theme().theme_name();
    MenuItem::Submenu(Menu {
        name: "Theme".into(),
        items: themes
            .iter()
            .map(|theme| {
                let checked = current_name == &theme.name;
                MenuItem::action(theme.name.clone(), SwitchTheme(theme.name.clone()))
                    .checked(checked)
            })
            .collect(),
        disabled: false,
    })
}
