use gpui::prelude::*;
use gpui::*;
use gpui_component::menu::AppMenuBar;
use gpui_component::{ActiveTheme, GlobalState, h_flex};

use crate::action::app::Quit;
use crate::action::query::{ExecuteQuery, NewQuery};
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};

pub struct Titlebar {
    app_menu: Entity<AppMenuBar>,
}

impl Titlebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let menus = build_menus();
        GlobalState::global_mut(cx).set_app_menus(menus.into_iter().map(|m| m.owned()).collect());
        let app_menu = AppMenuBar::new(cx);

        Self { app_menu }
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
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                div()
                    .id("menu-bar")
                    .key_context("menu-bar")
                    .flex_1()
                    .h_full()
                    .child(self.app_menu.clone()),
            )
    }
}

fn build_menus() -> Vec<Menu> {
    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Database", NewDatabase),
                MenuItem::action("Open File...", OpenFile),
                MenuItem::action("In-Memory Database", UseInMemory),
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
