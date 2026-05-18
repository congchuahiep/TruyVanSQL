use assets::AppIcon;
use engine::DatabaseKind;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::sidebar::Sidebar;
use gpui_component::spinner::Spinner;

use crate::action::sidebar::RefreshDatabase;
use crate::component::sidebar_menu_item::SidebarMenuItem;
use crate::connection::ConnectionStore;
use crate::connection::{ConnectionStatus, DatabaseConnection, DatabaseNode, SchemaNode};
use crate::panel::TabManager;
use crate::panel::TableViewerTab;

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
    let is_loading = conn.status == ConnectionStatus::Connecting;
    let is_online = conn.status == ConnectionStatus::Online;
    let is_offline = matches!(conn.status, ConnectionStatus::Error(_));

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
                .when(is_offline, |this| this.bg(cx.theme().red))
        })
        .context_menu(|popup, _, _| popup.menu("Refresh database", Box::new(RefreshDatabase)));

    let databases = conn.databases.clone();
    if !databases.is_empty() {
        let db_items: Vec<SidebarMenuItem> = databases
            .iter()
            .map(|db_node| {
                render_database_item(conn_entity.clone(), db_node, tab_manager.clone(), cx)
            })
            .collect();
        item = item.children(db_items);
    }

    item
}

fn render_database_item(
    conn_entity: Entity<DatabaseConnection>,
    db_node: &DatabaseNode,
    tab_manager: Entity<TabManager>,
    cx: &mut Context<Explorer>,
) -> SidebarMenuItem {
    let db_name = db_node.database.name.clone();

    let mut item = SidebarMenuItem::new(&db_name)
        .icon(AppIcon::Database)
        .double_click_to_expand(true)
        .loading(db_node.schemas_state.is_loading())
        .on_expand({
            let cloned_conn = conn_entity.clone();
            let db_name_for_expand = db_name.clone();
            move |_window, cx| {
                cloned_conn.update(cx, |c, cx| {
                    c.load_schemas(&db_name_for_expand, cx);
                });
            }
        });

    if let Some(schemas) = db_node.schemas_state.as_loaded() {
        let schema_items: Vec<SidebarMenuItem> = schemas
            .iter()
            .map(|schema_node| {
                render_schema_item(
                    conn_entity.clone(),
                    &db_name,
                    schema_node,
                    tab_manager.clone(),
                    cx,
                )
            })
            .collect();
        item = item.children(schema_items);
    }

    item
}

fn render_schema_item(
    conn_entity: Entity<DatabaseConnection>,
    db_name: &str,
    schema_node: &SchemaNode,
    tab_manager: Entity<TabManager>,
    _cx: &mut Context<Explorer>,
) -> SidebarMenuItem {
    let schema_name = schema_node.schema.name.clone();
    let schema_name_for_tables = schema_name.clone();
    let schema_name_for_views = schema_name.clone();
    let db_name_for_schema = db_name.to_string();

    let mut tables_group = {
        let conn_for_tables = conn_entity.clone();
        let db_for_tables = db_name_for_schema.clone();
        let schema_for_tables = schema_name_for_tables.clone();

        SidebarMenuItem::new("Tables")
            .icon(AppIcon::Folder)
            .loading(schema_node.tables_state.is_loading())
            .on_expand(move |_, cx| {
                conn_for_tables.update(cx, |c, cx| {
                    c.load_schema_tables(&db_for_tables, &schema_for_tables, cx);
                });
            })
    };

    let mut views_group = {
        let conn_for_views = conn_entity.clone();
        let db_for_views = db_name_for_schema.clone();
        let schema_for_views = schema_name_for_views.clone();

        SidebarMenuItem::new("Views")
            .icon(AppIcon::Folder)
            .loading(schema_node.views_state.is_loading())
            .on_expand(move |_, cx| {
                conn_for_views.update(cx, |c, cx| {
                    c.load_schema_views(&db_for_views, &schema_for_views, cx);
                });
            })
    };

    if let Some(tables) = schema_node.tables_state.as_loaded() {
        let table_items: Vec<SidebarMenuItem> = tables
            .iter()
            .map(|table| {
                let table_name = table.name.clone();
                let tab_manager_for_click = tab_manager.clone();
                let conn_for_table = conn_entity.clone();
                SidebarMenuItem::new(&table_name)
                    .icon(AppIcon::Table)
                    .indented(true)
                    .double_click_to_expand(false)
                    .on_double_click(move |_, window, cx| {
                        let tab = cx.new(|cx| {
                            TableViewerTab::new(
                                conn_for_table.clone(),
                                table_name.clone(),
                                window,
                                cx,
                            )
                        });
                        tab_manager_for_click.update(cx, |s: &mut TabManager, cx| {
                            s.open_tab(tab, cx);
                        });
                    })
            })
            .collect();
        tables_group = tables_group.children(table_items);
    }

    if let Some(views) = schema_node.views_state.as_loaded() {
        let view_items: Vec<SidebarMenuItem> = views
            .iter()
            .map(|view| {
                let view_name = view.name.clone();
                let tab_manager_for_click = tab_manager.clone();
                let conn_for_view = conn_entity.clone();
                SidebarMenuItem::new(&view_name)
                    .icon(AppIcon::Table)
                    .indented(true)
                    .double_click_to_expand(false)
                    .on_double_click(move |_, window, cx| {
                        let tab = cx.new(|cx| {
                            TableViewerTab::new(
                                conn_for_view.clone(),
                                view_name.clone(),
                                window,
                                cx,
                            )
                        });
                        tab_manager_for_click.update(cx, |s: &mut TabManager, cx| {
                            s.open_tab(tab, cx);
                        });
                    })
            })
            .collect();
        views_group = views_group.children(view_items);
    }

    let item = SidebarMenuItem::new(&schema_name)
        .icon(AppIcon::Schema)
        .children([tables_group, views_group]);

    item
}
