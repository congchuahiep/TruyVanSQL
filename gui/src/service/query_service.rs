use engine::{DatabaseConfig, QueryResult, SqlClient};
use gpui::*;
use gpui_component::input::InputState;

use crate::service::connection_service::ConnectionService;
use crate::state::OutputContent;

pub struct QueryService {
    pub sql_input: Entity<InputState>,
    pub output: OutputContent,
    pub is_executing: bool,
    pub connection: Entity<ConnectionService>,
}

impl QueryService {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        connection: Entity<ConnectionService>,
    ) -> Self {
        let sql_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("sql")
                .searchable(true)
                .line_number(true)
                .placeholder("Nhập SQL query tại đây...")
                .default_value("SELECT * FROM users;")
        });

        cx.observe(&connection, |this: &mut Self, _, cx| {
            this.reset_output(cx);
        })
        .detach();

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

        let db_url = self.connection.read(cx).db_url.clone();
        self.is_executing = true;
        self.output = OutputContent::Empty;
        cx.notify();

        let this = cx.entity();
        cx.spawn(async move |_, cx| {
            let result = Self::run_query(&db_url, &sql).await;
            this.update(cx, |service, cx| {
                service.is_executing = false;
                service.output = result;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn reset_output(&mut self, cx: &mut Context<Self>) {
        self.output = OutputContent::Empty;
        cx.notify();
    }

    async fn run_query(db_url: &str, sql: &str) -> OutputContent {
        let config = DatabaseConfig::sqlite(db_url);
        let client = match SqlClient::connect(config).await {
            Ok(c) => c,
            Err(e) => return OutputContent::Error(format!("Lỗi kết nối: {e}")),
        };
        match client.execute(sql).await {
            Ok(QueryResult::Query { columns, rows }) => OutputContent::Query { columns, rows },

            Ok(QueryResult::Execution {
                rows_affected,
                last_insert_rowid,
            }) => {
                let mut text = format!("{rows_affected} rows affected");
                if let Some(id) = last_insert_rowid {
                    text.push_str(&format!("\nlast_insert_rowid: {id}"));
                }
                OutputContent::Execution { text }
            }

            Err(e) => OutputContent::Error(format_error(&e)),
        }
    }
}

fn format_error(err: &engine::EngineError) -> String {
    match err {
        engine::EngineError::Connection(msg) => format!("Lỗi kết nối: {msg}"),
        engine::EngineError::QueryExecution(msg) => format!("Lỗi query: {msg}"),
        engine::EngineError::Schema(msg) => format!("Lỗi schema: {msg}"),
        engine::EngineError::UnsupportedDatabase(msg) => format!("DB không hỗ trợ: {msg}"),
    }
}
