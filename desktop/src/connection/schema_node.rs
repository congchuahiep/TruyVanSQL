use assets::AppIcon;
use engine::{SchemaBrief, SqlClient, TableBrief};
use gpui::*;

use crate::component::sidebar_menu_item::SidebarMenuItem;
use crate::connection::DatabaseConnection;
use crate::panel::{TabManager, TableViewerTab};
use crate::shared::LoadState;

/// Đại diện cho một node schema trong cây hiển thị.
/// Chứa thông tin về schema và danh sách các bảng, view bên trong.
/// Mỗi SchemaNode là một Entity tự quản lý việc load dữ liệu.
pub struct SchemaNode {
    pub schema: SchemaBrief,
    pub tables_state: LoadState<Vec<TableBrief>>,
    pub views_state: LoadState<Vec<TableBrief>>,
    client: SqlClient,
    db_name: String,
}

impl std::fmt::Debug for SchemaNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaNode")
            .field("schema", &self.schema)
            .field("tables_state", &self.tables_state)
            .field("views_state", &self.views_state)
            .field("db_name", &self.db_name)
            .finish()
    }
}

impl SchemaNode {
    /// Tạo SchemaNode mới với engine types.
    pub fn new(schema: SchemaBrief, client: SqlClient, db_name: String) -> Self {
        Self {
            schema,
            tables_state: LoadState::Idle,
            views_state: LoadState::Idle,
            client,
            db_name,
        }
    }

    /// Kiểm tra xem đang ở trạng thái idle (chưa load bao giờ)
    pub fn is_idle(&self) -> bool {
        self.tables_state.is_idle() && self.views_state.is_idle()
    }

    /// Load danh sách tables cho schema này.
    pub fn load_tables(&mut self, cx: &mut Context<Self>) {
        if !self.tables_state.is_idle() {
            return;
        }

        self.tables_state = LoadState::Loading;
        cx.notify();

        let client = self.client.clone();
        let schema_name = self.schema.name.clone();

        cx.spawn(async move |this, cx| {
            let tables = client.list_tables(&schema_name).await.unwrap_or_default();
            this.update(cx, |this, cx| {
                this.tables_state = LoadState::Loaded(tables);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Load danh sách views cho schema này.
    pub fn load_views(&mut self, cx: &mut Context<Self>) {
        if !self.views_state.is_idle() {
            return;
        }

        self.views_state = LoadState::Loading;
        cx.notify();

        let client = self.client.clone();
        let schema_name = self.schema.name.clone();

        cx.spawn(async move |this, cx| {
            let views = client.list_views(&schema_name).await.unwrap_or_default();
            this.update(cx, |this, cx| {
                this.views_state = LoadState::Loaded(views);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Render thành SidebarMenuItem cho Explorer sidebar.
    pub fn render_item(
        &self,
        schema_entity: Entity<Self>,
        tab_manager: Entity<TabManager>,
        conn_entity: Entity<DatabaseConnection>,
        cx: &App,
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
                    let table_name = table.name.clone();
                    let tab_manager_for_click = tab_manager.clone();
                    let conn_entity_for_tab = conn_entity.clone();
                    SidebarMenuItem::new(&table_name)
                        .icon(AppIcon::Table)
                        .indented(true)
                        .double_click_to_expand(false)
                        .on_double_click(move |_, window, cx| {
                            let tab = cx.new(|cx| {
                                TableViewerTab::new(
                                    conn_entity_for_tab.clone(),
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
                    let view_name = view.name.clone();
                    let tab_manager_for_click = tab_manager.clone();
                    let conn_entity_for_tab = conn_entity.clone();
                    SidebarMenuItem::new(&view_name)
                        .icon(AppIcon::Table)
                        .indented(true)
                        .double_click_to_expand(false)
                        .on_double_click(move |_, window, cx| {
                            let tab = cx.new(|cx| {
                                TableViewerTab::new(
                                    conn_entity_for_tab.clone(),
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
