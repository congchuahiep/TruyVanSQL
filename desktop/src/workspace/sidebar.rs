use gpui::prelude::*;
use gpui::*;
use gpui_component::sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem};
use gpui_component::{ActiveTheme, IconName};

use crate::action::sidebar::RefreshDatabase;
use crate::connection::model::ConnectionStatus;
use crate::connection::store::ConnectionStore;
use crate::tab_table_viewer::TableViewerTab;
use crate::workspace::tab_manager::TabManager;

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
        let theme = cx.theme();

        let tab_manager = self.tab_manager.clone();

        let store = self.connection_store.read(cx);
        let connections = store.connections().to_vec();

        Sidebar::new("explorer-sidebar")
            .w(px(260.0))
            .bg(theme.background)
            .border_r_1()
            .border_color(theme.border)
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
            .child(
                SidebarGroup::new("DATABASES").child(SidebarMenu::new().children(
                    connections.into_iter().map(|conn_entity| {
                        let conn = conn_entity.read(cx);
                        let conn_name = conn.name.clone();
                        let tables = conn.tables.clone();

                        let cloned_conn_1 = conn_entity.clone();
                        let cloned_conn_2 = conn_entity.clone();
                        let tab_manager_for_map = tab_manager.clone();

                        SidebarMenuItem::new(conn_name.clone())
                            .icon(IconName::Folder)
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cloned_conn_1.update(cx, |c, cx| {
                                    if c.tables.is_empty() && c.status == ConnectionStatus::Online {
                                        c.refresh_metadata(cx);
                                    }
                                });
                            }))
                            .context_menu(|popup, _, _| {
                                popup.menu("Refresh database", Box::new(RefreshDatabase))
                            })
                            .children(tables.into_iter().map(move |table| {
                                let table_name = table.name.clone();
                                let conn_for_click = cloned_conn_2.clone();
                                let tab_manager_for_click = tab_manager_for_map.clone();

                                SidebarMenuItem::new(table_name.clone())
                                    .icon(IconName::File)
                                    .on_click(move |_, window, cx| {
                                        // Khởi tạo TableViewerTab (Full DataGrid) khi click vào table
                                        let tab = cx.new(|cx| {
                                            TableViewerTab::new(
                                                conn_for_click.clone(),
                                                table_name.clone(),
                                                window,
                                                cx,
                                            )
                                        });
                                        tab_manager_for_click.update(cx, |s, cx| {
                                            s.open_tab(tab, cx);
                                        });
                                    })
                            }))
                    }),
                )),
            )
    }
}
