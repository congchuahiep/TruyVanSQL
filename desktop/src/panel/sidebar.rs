use assets::AppIcon;
use engine::DatabaseKind;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::Icon;
use gpui_component::sidebar::Sidebar;
use gpui_component::spinner::Spinner;

use crate::action::sidebar::RefreshDatabase;
use crate::component::sidebar_menu_item::SidebarMenuItem;
use crate::connection::ConnectionStatus;
use crate::connection::ConnectionStore;
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
            .children(
                connections
                    .into_iter()
                    .enumerate()
                    .map(|(_i, conn_entity)| {
                        let conn = conn_entity.read(cx);

                        let tables = conn.tables.clone();
                        let is_loading = conn.status == ConnectionStatus::Connecting;
                        let is_online = conn.status == ConnectionStatus::Online;
                        let is_offline = matches!(conn.status, ConnectionStatus::Error(_));

                        let db_icon = match conn.config.kind() {
                            DatabaseKind::Sqlite => AppIcon::Database,
                            DatabaseKind::Postgres => AppIcon::Postgres,
                        };

                        let cloned_conn_1 = conn_entity.clone();
                        let cloned_conn_2 = conn_entity.clone();
                        let tab_manager_for_map = self.tab_manager.clone();

                        SidebarMenuItem::new(&conn.name)
                            .icon(db_icon)
                            .expand_on_double_click(true)
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cloned_conn_1.update(cx, |c, cx| {
                                    if c.tables.is_empty() && c.status == ConnectionStatus::Online {
                                        c.refresh_metadata(cx);
                                    }
                                });
                            }))
                            .suffix(move |_, cx| {
                                div()
                                    .when(is_loading, |this| this.child(Spinner::new()).size_4())
                                    .when(is_online, |this| {
                                        this.size_1p5().rounded_full().bg(cx.theme().green).mr_1()
                                    })
                                    .when(is_offline, |this| {
                                        this.size_1p5().rounded_full().bg(cx.theme().red).mr_1()
                                    })
                            })
                            .context_menu(|popup, _, _| {
                                popup.menu("Refresh database", Box::new(RefreshDatabase))
                            })
                            .children(tables.into_iter().map(move |table| {
                                let table_name = table.name.clone();
                                let conn_for_click = cloned_conn_2.clone();
                                let tab_manager_for_click = tab_manager_for_map.clone();

                                SidebarMenuItem::new(table_name.clone())
                                    .icon(AppIcon::Table)
                                    .on_click(move |_, window, cx| {
                                        let tab = cx.new(|cx| {
                                            TableViewerTab::new(
                                                conn_for_click.clone(),
                                                table_name.clone(),
                                                window,
                                                cx,
                                            )
                                        });
                                        tab_manager_for_click.update(
                                            cx,
                                            |s: &mut TabManager, cx| {
                                                s.open_tab(tab, cx);
                                            },
                                        );
                                    })
                            }))
                            .collapsed(false)
                    }),
            )
    }
}
