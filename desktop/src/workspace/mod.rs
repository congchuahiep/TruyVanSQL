pub mod sidebar;
pub mod tab_bar;
pub mod tab_item;
pub mod tab_manager;
pub mod title_bar;

use gpui::prelude::*;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::action::query::{ExecuteQuery, NewQuery};
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::connection::store::ConnectionStore;
use crate::tab_sql_editor::SqlEditorTab;

use sidebar::Explorer;
use tab_bar::TabBar;
use tab_manager::TabManager;
use title_bar::Titlebar;

pub struct Workspace {
    focus_handle: FocusHandle,

    connection_store: Entity<ConnectionStore>,
    tab_manager: Entity<TabManager>,

    title_bar: Entity<Titlebar>,
    sidebar: Entity<Explorer>,
    tab_bar: Entity<TabBar>,
}

impl Workspace {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let connection_store = cx.new(|cx| ConnectionStore::new(cx));
        let tab_manager = cx.new(|cx| TabManager::new(cx));

        let title_bar = cx.new(|cx| Titlebar::new(cx));
        let sidebar = cx.new(|cx| Explorer::new(connection_store.clone(), tab_manager.clone(), cx));
        let tab_bar = cx.new(|cx| TabBar::new(tab_manager.clone(), cx));

        cx.observe(&tab_manager, |_, _, cx| cx.notify()).detach();

        Self {
            focus_handle,
            connection_store,
            tab_manager,
            title_bar,
            sidebar,
            tab_bar,
        }
    }

    fn on_new_database(&mut self, _action: &NewDatabase, _w: &mut Window, cx: &mut Context<Self>) {
        let store = self.connection_store.clone();
        cx.spawn(async move |_, cx| {
            let dialog = rfd::AsyncFileDialog::new()
                .set_title("Tạo database mới")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"]);

            if let Some(file) = dialog.save_file().await {
                let path = file.path().to_string_lossy().to_string();
                let name = file.file_name();
                let config = engine::DatabaseConfig::sqlite(format!("sqlite:{}", path));

                store.update(cx, |w, cx| {
                    w.add_connection(name, config, cx);
                });
            }
        })
        .detach();
    }

    fn on_open_file(&mut self, _action: &OpenFile, _w: &mut Window, cx: &mut Context<Self>) {
        let store = self.connection_store.clone();
        cx.spawn(async move |_, cx| {
            let dialog = rfd::AsyncFileDialog::new()
                .set_title("Mở database")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                .add_filter("All Files", &["*"]);

            if let Some(file) = dialog.pick_file().await {
                let path = file.path().to_string_lossy().to_string();
                let name = file.file_name();
                let config = engine::DatabaseConfig::sqlite(format!("sqlite:{}", path));

                store.update(cx, |w, cx| {
                    w.add_connection(name, config, cx);
                });
            }
        })
        .detach();
    }

    fn on_in_memory(&mut self, _action: &UseInMemory, _w: &mut Window, cx: &mut Context<Self>) {
        self.connection_store.update(cx, |w, cx| {
            w.add_connection(
                "In-Memory".into(),
                engine::DatabaseConfig::sqlite("sqlite::memory:"),
                cx,
            );
        });
    }

    fn on_new_query(&mut self, _action: &NewQuery, w: &mut Window, cx: &mut Context<Self>) {
        let store = self.connection_store.read(cx);
        let connections = store.connections();

        if let Some(first_conn) = connections.first() {
            let conn_clone = first_conn.clone();
            self.tab_manager.update(cx, |s, cx| {
                let tab = cx.new(|cx| SqlEditorTab::new(conn_clone, w, cx));
                s.open_tab(tab, cx);
            });
        }
    }

    fn on_execute(&mut self, _action: &ExecuteQuery, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_tab) = self.tab_manager.read(cx).active_tab() {
            if let Ok(editor_tab) = active_tab.view().downcast::<SqlEditorTab>() {
                editor_tab.update(cx, |tab, cx| {
                    tab.execute(cx);
                });
            }
        }
    }

    fn register_actions(&self, el: Div, cx: &mut Context<Self>) -> Div {
        el.key_context("app")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_new_database))
            .on_action(cx.listener(Self::on_open_file))
            .on_action(cx.listener(Self::on_in_memory))
            .on_action(cx.listener(Self::on_new_query))
            .on_action(cx.listener(Self::on_execute))
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.tab_manager.read(cx).active_tab();

        self.register_actions(v_flex(), cx)
            .size_full()
            .id("workspace")
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .child(self.sidebar.clone())
                    .child(
                        v_flex()
                            .flex_1()
                            .size_full()
                            .min_w_0()
                            .min_h_0()
                            .border_t_1()
                            .border_l_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().background.alpha(0.4))
                            .child(self.tab_bar.clone())
                            .child(div().flex_1().flex_grow().min_w_0().min_h_0().child(
                                if let Some(tab) = active_tab {
                                    tab.view().into_any_element()
                                } else {
                                    v_flex()
                                        .flex_grow()
                                        .size_full()
                                        .items_center()
                                        .justify_center()
                                        .child("Chưa có tab nào được mở.")
                                        .child("Chọn một database ở Explorer để bắt đầu.")
                                        .into_any_element()
                                },
                            )),
                    ),
            )
    }
}
