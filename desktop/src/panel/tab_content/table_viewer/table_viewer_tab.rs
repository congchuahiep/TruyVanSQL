use assets::AppIcon;
use engine::{QueryResult, SqlClient};
use gpui::*;
use gpui_component::v_flex;
use std::any::Any;

use crate::panel::{TabInfo, TabItem};
use crate::shared::smart_data_grid::SmartDataGrid;

/// Tab chuyên dụng để hiển thị toàn màn hình DataGrid (Table Viewer)
pub struct TableViewerTab {
    table_name: SharedString,
    client: SqlClient,
    grid: Entity<SmartDataGrid>,
}

impl TableViewerTab {
    pub fn new(
        client: SqlClient,
        table_name: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let grid = cx.new(|cx| SmartDataGrid::new(client.clone(), window, cx));

        let tab = Self {
            table_name: table_name.into(),
            client: client.clone(),
            grid,
        };

        tab.load_data(cx);
        tab
    }

    fn load_data(&self, cx: &mut Context<Self>) {
        let table_name = self.table_name.clone();
        let grid_entity = self.grid.clone();
        let client = self.client.clone();
        let query = format!("SELECT * FROM \"{}\" LIMIT 1000", table_name);

        grid_entity.update(cx, |grid, cx| {
            grid.table.update(cx, |table, cx| {
                table.delegate_mut().state.is_loading = true;
                cx.notify();
            });
        });

        cx.spawn(async move |_, cx| {
            let mut pks = Vec::new();
            if let Ok(info) = client.get_table_info(&table_name).await {
                pks = info.primary_key.columns;
            }

            let result = client.execute(&query).await;

            grid_entity.update(cx, |grid, cx| {
                if let Ok(QueryResult::Query { columns, rows }) = result {
                    println!("Columns: {:?}, Rows: {}", columns, rows.len());
                    grid.set_data(columns, rows, cx);
                    grid.set_metadata(Some(table_name.clone()), pks, cx);
                } else if let Err(e) = result {
                    grid.table.update(cx, |table, _cx| {
                        table.delegate_mut().state.error = Some(e.to_string().into());
                    });
                    eprintln!("{}", e);
                }

                grid.table.update(cx, |table, cx| {
                    table.delegate_mut().state.is_loading = false;
                    cx.notify();
                });
            });
        })
        .detach();
    }
}

impl TabItem for TableViewerTab {
    fn tab_info(&self, cx: &App) -> TabInfo {
        let state = &self.grid.read(cx).table.read(cx).delegate().state;

        TabInfo {
            title: self.table_name.clone().into(),
            is_dirty: state.has_pending_changes(),
            is_loading: state.is_loading,
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
