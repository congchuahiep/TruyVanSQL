use assets::AppIcon;
use engine::DatabaseKind;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::sidebar::Sidebar;

use crate::action::sidebar::RefreshDatabase;
use crate::component::sidebar_menu_item::SidebarMenuItem;
use crate::connection::ConnectionStore;
use crate::connection::{ConnectionStatus, DatabaseConnection};
use crate::panel::TabManager;

pub struct Explorer {
    connection_store: Entity<ConnectionStore>,
    tab_manager: Entity<TabManager>,
}

impl Explorer {
    pub fn new(
        connection_store: Entity<ConnectionStore>,
        tab_manager: Entity<TabManager>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&connection_store, |_, _, cx| cx.notify())
            .detach();

        Self {
            connection_store,
            tab_manager,
        }
    }
}

impl Render for Explorer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.connection_store.read(cx);
        let connections = store.connections().to_vec();

        Sidebar::new("explorer-sidebar")
            .w(px(260.0))
            .border_0()
            .header(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .child("EXPLORER")
                            .font_weight(FontWeight::BOLD)
                            .text_xs(),
                    ),
            )
            .children(connections.iter().map(|conn_entity| {
                render_connection_item(conn_entity.clone(), self.tab_manager.clone(), cx)
            }))
    }
}

fn render_connection_item(
    conn_entity: Entity<DatabaseConnection>,
    tab_manager: Entity<TabManager>,
    cx: &mut Context<Explorer>,
) -> SidebarMenuItem {
    let conn = conn_entity.read(cx);
    let is_loading = conn.status.is_connecting();
    let is_online = conn.status.is_connected();
    let is_offline = conn.status.is_disconnected();
    let is_error = conn.status.is_error();

    let db_icon = match conn.config.kind() {
        DatabaseKind::Sqlite => AppIcon::Sqlite,
        DatabaseKind::Postgres => AppIcon::Postgres,
    };

    let mut item = SidebarMenuItem::new(&conn.name)
        .icon(db_icon)
        .loading(is_loading)
        .double_click_to_expand(true)
        .on_expand({
            let cloned_conn = conn_entity.clone();
            move |_window, cx| {
                cloned_conn.update(cx, |c, cx| {
                    if c.databases.is_empty() && c.status == ConnectionStatus::Online {
                        c.refresh_databases(cx);
                    }
                });
            }
        })
        .suffix(move |_, cx| {
            div()
                .size_1p5()
                .rounded_full()
                .mr_1()
                .bg(cx.theme().transparent)
                .when(is_loading, |this| this.bg(cx.theme().yellow))
                .when(is_online, |this| this.bg(cx.theme().green))
                .when(is_offline, |this| this.bg(cx.theme().muted))
                .when(is_error, |this| this.bg(cx.theme().red))
        })
        .context_menu(|popup, _, _| popup.menu("Refresh database", Box::new(RefreshDatabase)));

    let db_entities = conn.databases.clone();
    let _ = conn;

    if !db_entities.is_empty() {
        let db_items: Vec<SidebarMenuItem> = db_entities
            .iter()
            .map(|db_entity| {
                let tab_manager_clone = tab_manager.clone();
                let conn_entity_clone = conn_entity.clone();
                db_entity.read(cx).render_item(
                    db_entity.clone(),
                    tab_manager_clone,
                    conn_entity_clone,
                    cx,
                )
            })
            .collect();
        item = item.children(db_items);
    }

    item
}
