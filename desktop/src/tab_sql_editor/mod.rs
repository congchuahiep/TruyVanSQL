pub mod editor;
pub mod results;
pub mod session;
pub mod state;
pub mod table_delegate;
pub mod toolbar;

use assets::AppIcon;
use gpui::*;
use gpui_component::v_flex;
use std::any::Any;

use crate::connection::model::DatabaseConnection;
use crate::workspace::tab_item::{TabInfo, TabItem};

use editor::QueryEditor;
use results::QueryResults;
use session::QuerySession;
use toolbar::QueryToolbar;

/// View chính của tính năng SQL Editor Tab.
/// Nó gom nhóm Logic (Session) và các UI Component (Editor, Toolbar, Results).
pub struct SqlEditorTab {
    session: Entity<QuerySession>,
    editor: Entity<QueryEditor>,
    toolbar: Entity<QueryToolbar>,
    results: Entity<QueryResults>,
}

impl SqlEditorTab {
    pub fn new(
        connection: Entity<DatabaseConnection>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let session = cx.new(|cx| QuerySession::new(window, cx, connection));

        let sql_input = session.read(cx).sql_input.clone();
        let editor = cx.new(|_| QueryEditor::new(sql_input));
        let toolbar = cx.new(|cx| QueryToolbar::new(session.clone(), cx));
        let results = cx.new(|cx| QueryResults::new(session.clone(), window, cx));

        cx.observe(&session, |_, _, cx| cx.notify()).detach();

        Self {
            session,
            editor,
            toolbar,
            results,
        }
    }

    pub fn execute(&mut self, cx: &mut Context<Self>) {
        self.session.update(cx, |s, cx| s.execute(cx));
    }
}

impl TabItem for SqlEditorTab {
    fn tab_info(&self, cx: &App) -> TabInfo {
        let conn_name = self.session.read(cx).connection.read(cx).name.clone();
        TabInfo {
            title: format!("SQL: {}", conn_name).into(),
            is_dirty: false,
            icon: AppIcon::FileSql,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Render for SqlEditorTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(self.editor.clone())
            .child(self.toolbar.clone())
            .child(self.results.clone())
    }
}
