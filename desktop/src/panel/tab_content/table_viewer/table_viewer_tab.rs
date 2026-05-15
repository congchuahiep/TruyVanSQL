use assets::AppIcon;
use engine::QueryResult;
use gpui::*;
use gpui_component::v_flex;
use std::any::Any;

use crate::connection::DatabaseConnection;
use crate::panel::{TabInfo, TabItem};
use crate::shared::smart_data_grid::SmartDataGrid;

/// Tab chuyên dụng để hiển thị toàn màn hình DataGrid (Table Viewer)
pub struct TableViewerTab {
    table_name: String,
    #[allow(dead_code)]
    connection: Entity<DatabaseConnection>,
    grid: Entity<SmartDataGrid>,
}

impl TableViewerTab {
    pub fn new(
        connection: Entity<DatabaseConnection>,
        table_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let grid = cx.new(|cx| SmartDataGrid::new(connection.clone(), window, cx));

        let tab = Self {
            table_name: table_name.clone(),
            connection: connection.clone(),
            grid,
        };

        tab.load_data(table_name, connection, tab.grid.clone(), cx);
        tab
    }

    fn load_data(
        &self,
        table_name: String,
        connection: Entity<DatabaseConnection>,
        grid_entity: Entity<SmartDataGrid>,
        cx: &mut Context<Self>,
    ) {
        let conn = connection.read(cx);
        let client = if let Some(c) = &conn.client {
            c.clone()
        } else {
            return;
        };

        let query = format!("SELECT * FROM \"{}\" LIMIT 1000", table_name);

        cx.spawn(async move |_, cx| {
            let mut pks = Vec::new();
            if let Ok(info) = client.get_table_info(&table_name).await {
                pks = info.primary_key.columns;
            }

            let result = client.execute(&query).await;

            grid_entity.update(cx, |grid, cx| {
                if let Ok(QueryResult::Query { columns, rows }) = result {
                    grid.set_data(columns, rows, cx);
                    grid.set_metadata(Some(table_name.clone()), pks, cx);
                } else if let Err(e) = result {
                    eprintln!("TableViewerTab Lỗi: {}", e);
                }
            });
        })
        .detach();
    }
}

impl TabItem for TableViewerTab {
    fn tab_info(&self, cx: &App) -> TabInfo {
        TabInfo {
            title: self.table_name.clone().into(),
            is_dirty: self
                .grid
                .read(cx)
                .table
                .read(cx)
                .delegate()
                .state
                .has_pending_changes(),
            icon: AppIcon::Table,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Render for TableViewerTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(self.grid.clone())
    }
}