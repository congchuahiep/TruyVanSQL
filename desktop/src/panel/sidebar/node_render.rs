use crate::component::sidebar_menu_item::SidebarMenuItem;
use crate::connection::{DatabaseNode, SchemaNode};
use crate::panel::{TabManager, TableViewerTab};
use assets::AppIcon;
use gpui::*;

impl DatabaseNode {
    /// Render thành SidebarMenuItem cho Explorer sidebar.
    pub fn render_item(
        &self,
        db_entity: Entity<Self>,
        tab_manager: Entity<TabManager>,
        cx: &App,
    ) -> SidebarMenuItem {
        let db_name = self.database.name.clone();
        let schemas_loading = self.schemas_state.is_loading();
        let schemas = if let Some(schemas) = self.schemas_state.as_loaded() {
            schemas.clone()
        } else {
            Vec::new()
        };

        let mut item = SidebarMenuItem::new(&db_name)
            .icon(AppIcon::Database)
            .double_click_to_expand(true)
            .loading(schemas_loading)
            .on_expand({
                let db_entity = db_entity.clone();
                move |_window, cx| {
                    db_entity.update(cx, |db, cx| {
                        db.load_schemas(cx);
                    });
                }
            });

        if !schemas.is_empty() {
            let schema_items: Vec<SidebarMenuItem> = schemas
                .iter()
                .map(|schema_entity| {
                    schema_entity.read(cx).render_item(
                        schema_entity.clone(),
                        tab_manager.clone(),
                        cx,
                    )
                })
                .collect();
            item = item.children(schema_items);
        }

        item
    }
}

impl SchemaNode {
    /// Render thành SidebarMenuItem cho Explorer sidebar.
    pub fn render_item(
        &self,
        schema_entity: Entity<Self>,
        tab_manager: Entity<TabManager>,
        _cx: &App,
    ) -> SidebarMenuItem {
        let schema_name = self.schema.name.clone();
        let tables_state = self.tables_state.clone();
        let views_state = self.views_state.clone();

        let mut tables_group = {
            let schema_entity = schema_entity.clone();
            SidebarMenuItem::new("Tables")
                .icon(AppIcon::Folder)
                .loading(self.tables_state.is_loading())
                .on_expand(move |_, cx| {
                    schema_entity.update(cx, |s, cx| {
                        s.load_tables(cx);
                    });
                })
        };

        let mut views_group = {
            let schema_entity = schema_entity.clone();
            SidebarMenuItem::new("Views")
                .icon(AppIcon::Folder)
                .loading(self.views_state.is_loading())
                .on_expand(move |_, cx| {
                    schema_entity.update(cx, |s, cx| {
                        s.load_views(cx);
                    });
                })
        };

        if let Some(tables) = tables_state.as_loaded() {
            let table_items: Vec<SidebarMenuItem> = tables
                .iter()
                .map(|table| {
                    let table_name: SharedString = table.name.clone().into();
                    let tab_manager_for_click = tab_manager.clone();
                    let client_for_tab = self.client.clone();
                    SidebarMenuItem::new(&table_name)
                        .icon(AppIcon::Table)
                        .indented(true)
                        .double_click_to_expand(false)
                        .on_double_click(move |_, window, cx| {
                            let tab = cx.new(|cx| {
                                TableViewerTab::new(
                                    client_for_tab.clone(),
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

        if let Some(views) = views_state.as_loaded() {
            let view_items: Vec<SidebarMenuItem> = views
                .iter()
                .map(|view| {
                    let view_name: SharedString = view.name.clone().into();
                    let tab_manager_for_click = tab_manager.clone();
                    let client_for_tab = self.client.clone();
                    SidebarMenuItem::new(&view_name)
                        .icon(AppIcon::Table)
                        .indented(true)
                        .double_click_to_expand(false)
                        .on_double_click(move |_, window, cx| {
                            let tab = cx.new(|cx| {
                                TableViewerTab::new(
                                    client_for_tab.clone(),
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

        SidebarMenuItem::new(&schema_name)
            .icon(AppIcon::Schema)
            .children([tables_group, views_group])
    }
}
