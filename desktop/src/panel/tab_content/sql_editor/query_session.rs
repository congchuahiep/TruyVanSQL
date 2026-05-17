use crate::connection::DatabaseConnection;
use crate::panel::tab_content::sql_editor::output_content::OutputContent;
use engine::QueryResult;
use gpui::*;
use gpui_component::input::InputState;

/// Quản lý trạng thái và logic của một phiên chạy SQL.
pub struct QuerySession {
    pub sql_input: Entity<InputState>,
    pub output: OutputContent,
    pub is_executing: bool,
    pub connection: Entity<DatabaseConnection>,
}

impl QuerySession {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        connection: Entity<DatabaseConnection>,
    ) -> Self {
        let sql_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("sql")
                .line_number(true)
                .placeholder("Nhập SQL query tại đây...")
                .default_value("SELECT * FROM sqlite_master;")
        });

        cx.observe(&connection, |_, _, cx| cx.notify()).detach();

        Self {
            sql_input,
            output: OutputContent::Empty,
            is_executing: false,
            connection,
        }
    }

    pub fn execute(&mut self, cx: &mut Context<Self>) {
        let sql = self.sql_input.read(cx).value().to_string();
        if sql.trim().is_empty() {
            return;
        }

        let conn = self.connection.read(cx);
        if conn.status != crate::connection::ConnectionStatus::Online {
            self.output = OutputContent::Error("Database chưa kết nối.".into());
            cx.notify();
            return;
        }

        let client = if let Some(c) = &conn.client {
            c.clone()
        } else {
            return;
        };

        self.is_executing = true;
        self.output = OutputContent::Empty;
        cx.notify();

        let this = cx.entity();
        let conn_entity = self.connection.clone();

        cx.spawn(async move |_, cx| {
            let result = match client.execute(&sql).await {
                Ok(QueryResult::Query { columns, rows }) => OutputContent::Query { columns, rows },
                Ok(QueryResult::Execution {
                    rows_affected,
                    last_insert_rowid,
                }) => {
                    let mut text = format!("{} rows affected", rows_affected);
                    if let Some(id) = last_insert_rowid {
                        text.push_str(&format!("\nlast_insert_rowid: {}", id));
                    }
                    OutputContent::Execution { text }
                }
                Err(e) => OutputContent::Error(e.to_string()),
            };

            this.update(cx, |service, cx| {
                service.is_executing = false;
                service.output = result;
                cx.notify();
            });

            if is_ddl(&sql) {
                conn_entity.update(cx, |c, cx| c.refresh_metadata(cx));
            }
        })
        .detach();
    }
}

fn is_ddl(sql: &str) -> bool {
    let s = sql.trim_start().to_uppercase();
    s.starts_with("CREATE")
        || s.starts_with("DROP")
        || s.starts_with("ALTER")
        || s.starts_with("RENAME")
}