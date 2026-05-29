use assets::AppIcon;
use engine::SqlClient;
use gpui::*;
use gpui_component::v_flex;
use std::any::Any;

use crate::panel::{TabInfo, TabItem};
use crate::shared::smart_data_grid::{GridDataSource, SmartDataGrid};

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
        let table_name = table_name.into();

        let grid = cx.new(|cx| {
            SmartDataGrid::new(
                client.clone(),
                GridDataSource::Table {
                    source_table: table_name.clone(),
                },
                window,
                cx,
            )
        });

        let tab = Self {
            table_name: table_name,
            client: client.clone(),
            grid,
        };

        tab
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
