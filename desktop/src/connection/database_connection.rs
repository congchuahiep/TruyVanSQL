use engine::{DatabaseConfig, SqlClient, TableBrief};
use gpui::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Online,
    Error(String),
}

pub struct DatabaseConnection {
    pub name: SharedString,
    pub config: DatabaseConfig,
    pub status: ConnectionStatus,
    pub tables: Vec<TableBrief>,
    pub views: Vec<String>,
    pub client: Option<SqlClient>,
}

impl DatabaseConnection {
    pub fn new(name: SharedString, config: DatabaseConfig, _cx: &mut Context<Self>) -> Self {
        Self {
            name,
            config,
            status: ConnectionStatus::Disconnected,
            tables: Vec::new(),
            views: Vec::new(),
            client: None,
        }
    }

    pub fn connect(&mut self, cx: &mut Context<Self>) {
        if self.status == ConnectionStatus::Connecting || self.status == ConnectionStatus::Online {
            return;
        }

        self.status = ConnectionStatus::Connecting;
        cx.notify();

        let config = self.config.clone();
        cx.spawn(
            async move |this, cx| match SqlClient::connect(config).await {
                Ok(client) => {
                    let tables = client.list_tables().await.unwrap_or_default();
                    let views = client.list_views().await.unwrap_or_default();

                    this.update(cx, |this, cx| {
                        this.client = Some(client);
                        this.tables = tables;
                        this.views = views;
                        this.status = ConnectionStatus::Online;
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    this.update(cx, |this, cx| {
                        this.status = ConnectionStatus::Error(e.to_string());
                        cx.notify();
                    })
                    .ok();
                }
            },
        )
        .detach();
    }

    pub fn refresh_metadata(&mut self, cx: &mut Context<Self>) {
        let client = if let Some(c) = &self.client {
            c.clone()
        } else {
            return;
        };

        cx.spawn(async move |this, cx| {
            let tables = client.list_tables().await.unwrap_or_default();
            let views = client.list_views().await.unwrap_or_default();

            this.update(cx, |this, cx| {
                this.tables = tables;
                this.views = views;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.client = None;
        self.status = ConnectionStatus::Disconnected;
        self.tables.clear();
        self.views.clear();
        cx.notify();
    }
}
