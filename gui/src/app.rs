use std::time::Duration;

use gpui::prelude::*;
use gpui::*;
use gpui_component::v_flex;

use crate::action::query::ExecuteQuery;
use crate::action::toolbar::{NewDatabase, OpenFile, UseInMemory};
use crate::panel::execute_bar::ExecuteBar;
use crate::panel::output_panel::OutputPanel;
use crate::panel::sql_editor::SqlEditor;
use crate::panel::titlebar::Titlebar;
use crate::service::connection_service::ConnectionService;
use crate::service::query_service::QueryService;
use crate::util::DebouncedDelay;
use crate::window_state::WindowState;

pub struct AppView {
    focus_handle: FocusHandle,

    connection: Entity<ConnectionService>,
    query: Entity<QueryService>,

    titlebar: Entity<Titlebar>,
    sql_editor: Entity<SqlEditor>,
    execute_bar: Entity<ExecuteBar>,
    output_panel: Entity<OutputPanel>,

    _save_bounds_debounce: DebouncedDelay<Self>,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let connection = cx.new(|cx| ConnectionService::new(cx));
        let query = cx.new(|cx| QueryService::new(window, cx, connection.clone()));

        let titlebar = cx.new(|cx| Titlebar::new(connection.clone(), cx));
        let sql_input = query.read(cx).sql_input.clone();
        let sql_editor = cx.new(|_cx| SqlEditor::new(sql_input));
        let execute_bar = cx.new(|cx| ExecuteBar::new(query.clone(), window, cx));
        let output_panel = cx.new(|cx| OutputPanel::new(query.clone(), window, cx));

        cx.observe_window_bounds(window, |this, window, cx| {
            let window_bounds = window.window_bounds();
            this._save_bounds_debounce
                .fire_new(Duration::from_secs(1), cx, move |_this, _cx| {
                    let state = WindowState { window_bounds };
                    state.save();
                    Task::ready(())
                });
        })
        .detach();

        Self {
            focus_handle,
            titlebar,
            connection,
            query,
            sql_editor,
            execute_bar,
            output_panel,
            _save_bounds_debounce: DebouncedDelay::new(),
        }
    }

    fn on_new_database(&mut self, _action: &NewDatabase, _w: &mut Window, cx: &mut Context<Self>) {
        println!("on_new_database");
        self.connection.update(cx, |c, cx| c.new_database(cx));
    }

    fn on_open_file(&mut self, _action: &OpenFile, _w: &mut Window, cx: &mut Context<Self>) {
        println!("on_open_file");
        self.connection.update(cx, |c, cx| c.open_file(cx));
    }

    fn on_in_memory(&mut self, _action: &UseInMemory, _w: &mut Window, cx: &mut Context<Self>) {
        self.connection.update(cx, |c, cx| c.use_in_memory(cx));
    }

    fn on_execute(&mut self, _acion: &ExecuteQuery, _w: &mut Window, cx: &mut Context<Self>) {
        self.query.update(cx, |q, cx| q.execute(cx));
    }

    fn register_actions(&self, el: Div, cx: &mut Context<Self>) -> Div {
        el.key_context("app")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_new_database))
            .on_action(cx.listener(Self::on_open_file))
            .on_action(cx.listener(Self::on_in_memory))
            .on_action(cx.listener(Self::on_execute))
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.register_actions(v_flex(), cx)
            .size_full()
            .id("app-view")
            .size_full()
            .child(self.titlebar.clone())
            .child(self.sql_editor.clone())
            .child(self.execute_bar.clone())
            .child(self.output_panel.clone())
    }
}
