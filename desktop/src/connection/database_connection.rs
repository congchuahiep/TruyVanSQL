use engine::{DatabaseConfig, SqlClient};
use gpui::*;

use crate::shared::LoadState;

use super::{DatabaseNode, SchemaNode};

/// Trạng thái kết nối đến server cơ sở dữ liệu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Online,
    Error(String),
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Online)
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionStatus::Connecting)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self, ConnectionStatus::Disconnected)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ConnectionStatus::Error(_))
    }
}

/// Đại diện cho một kết nối đến cơ sở dữ liệu, bao gồm tên, cấu hình, trạng thái và danh sách
/// các cơ sở dữ liệu.
pub struct DatabaseConnection {
    pub name: SharedString,
    pub config: DatabaseConfig,
    pub status: ConnectionStatus,
    pub databases: Vec<Entity<DatabaseNode>>,
    pub client: Option<SqlClient>,
}

impl DatabaseConnection {
    pub fn new(name: SharedString, config: DatabaseConfig, _cx: &mut Context<Self>) -> Self {
        Self {
            name,
            config,
            status: ConnectionStatus::Disconnected,
            databases: Vec::new(),
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
                    let databases = client.list_databases().await.unwrap_or_default();

                    this.update(cx, |this, cx| {
                        this.client = Some(client.clone());

                        let db_entities: Vec<Entity<DatabaseNode>> = databases
                            .into_iter()
                            .map(|db| {
                                let entity = cx.new(|_| DatabaseNode::new(db, client.clone()));
                                cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                                entity
                            })
                            .collect();

                        this.databases = db_entities;
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

    pub fn switch_database(&mut self, db_name: &str, cx: &mut Context<Self>) {
        let mut config = self.config.clone();
        let db_name_owned = db_name.to_string();
        config.set_database(&db_name_owned);

        self.status = ConnectionStatus::Connecting;
        self.databases.clear();
        cx.notify();

        let config_clone = config.clone();
        let db_name_for_async = db_name_owned;
        cx.spawn(
            async move |this, cx| match SqlClient::connect(config_clone).await {
                Ok(client) => {
                    let databases = client.list_databases().await.unwrap_or_default();

                    this.update(cx, |this, cx| {
                        this.client = Some(client.clone());
                        this.config.set_database(&db_name_for_async);

                        let db_entities: Vec<Entity<DatabaseNode>> = databases
                            .into_iter()
                            .map(|db| {
                                let entity = cx.new(|_| DatabaseNode::new(db, client.clone()));
                                cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                                entity
                            })
                            .collect();

                        this.databases = db_entities;
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

    pub fn refresh_databases(&mut self, cx: &mut Context<Self>) {
        let client = if let Some(c) = &self.client {
            c.clone()
        } else {
            return;
        };

        cx.spawn(async move |this, cx| {
            let databases = client.list_databases().await.unwrap_or_default();

            this.update(cx, |this, cx| {
                let db_entities: Vec<Entity<DatabaseNode>> = databases
                    .into_iter()
                    .map(|db| {
                        let entity = cx.new(|_| DatabaseNode::new(db, client.clone()));
                        cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                        entity
                    })
                    .collect();

                this.databases = db_entities;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.client = None;
        self.status = ConnectionStatus::Disconnected;
        self.databases.clear();
        cx.notify();
    }
}
