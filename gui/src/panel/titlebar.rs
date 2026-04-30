use gpui::prelude::*;
use gpui::*;
use gpui_component::label::Label;
use gpui_component::menu::AppMenuBar;
use gpui_component::{ActiveTheme, GlobalState, h_flex};

use crate::action::app::Quit;
use crate::action::query::ExecuteQuery;
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::service::connection_service::ConnectionService;

pub struct Titlebar {
    app_menu: Entity<AppMenuBar>,
    connection: Entity<ConnectionService>,
}

impl Titlebar {
    pub fn new(connection: Entity<ConnectionService>, cx: &mut Context<Self>) -> Self {
        let menus = build_menus();
        GlobalState::global_mut(cx).set_app_menus(menus.into_iter().map(|m| m.owned()).collect());
        let app_menu = AppMenuBar::new(cx);

        cx.observe(&connection, |_, _, cx| cx.notify()).detach();

        Self {
            app_menu,
            connection,
        }
    }
}

impl Render for Titlebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let db_path = self.connection.read(cx).db_path.clone();

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
            .child(
                div().px_2().flex_shrink_0().child(
                    Label::new(db_path)
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                ),
            )
    }
}

/// Xây dựng menu definitions cho ứng dụng.
///
/// Menu structure:
/// - **File**: New Database, Open File, In-Memory, separator, Quit
/// - **Query**: Run Query (Ctrl+Enter)
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
            items: vec![MenuItem::action("Run Query", ExecuteQuery)],
            disabled: false,
        },
        Menu {
            name: "Window".into(),
            items: vec![],
            disabled: false,
        },
    ]
}
