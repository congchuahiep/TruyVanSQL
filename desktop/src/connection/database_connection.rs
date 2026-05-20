use engine::{DatabaseConfig, DatabaseKind, SqlClient};
use gpui::*;

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

/// Phần tử con trực tiếp của một kết nối trong cây sidebar.
#[derive(Debug)]
pub enum ConnectionChildren {
    /// Postgres/MySQL — có database → schema → table
    Databases(Vec<Entity<DatabaseNode>>),
    /// SQLite — không có database, chỉ có schema → table
    Schemas(Vec<Entity<SchemaNode>>),
}

impl Default for ConnectionChildren {
    fn default() -> Self {
        Self::Schemas(vec![])
    }
}

/// Đại diện cho một kết nối đến cơ sở dữ liệu, bao gồm tên, cấu hình, trạng thái và danh sách
/// các cơ sở dữ liệu.
pub struct DatabaseConnection {
    pub name: SharedString,
    pub config: DatabaseConfig,
    pub status: ConnectionStatus,
    pub children: ConnectionChildren,
    pub client: Option<SqlClient>,
}

impl DatabaseConnection {
    pub fn new(name: SharedString, config: DatabaseConfig, _cx: &mut Context<Self>) -> Self {
        Self {
            name,
            config,
            status: ConnectionStatus::Disconnected,
            children: ConnectionChildren::default(),
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
                    this.update(cx, |this, cx| {
                        this.client = Some(client.clone());
                        this.status = ConnectionStatus::Online;
                        cx.notify();

                        this.load_children(cx);
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

    pub fn switch_database(&mut self, _db_name: &str, _cx: &mut Context<Self>) {
        todo!("Cần triển khai phương thức này")
    }

    pub fn refresh_databases(&mut self, cx: &mut Context<Self>) {
        self.load_children(cx);
    }

    pub fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.client = None;
        self.status = ConnectionStatus::Disconnected;
        self.children = ConnectionChildren::default();
        cx.notify();
    }

    /// Tải danh sách children (databases hoặc schemas) tùy loại DB.
    /// Được gọi từ connect() và refresh_databases().
    fn load_children(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };

        let kind = self.config.kind();
        cx.spawn(async move |this, cx| match kind {
            DatabaseKind::Sqlite => {
                let result = client.list_schemas().await;
                this.update(cx, |this, cx| {
                    match result {
                        Ok(schemas) => {
                            let children = schemas
                                .into_iter()
                                .map(|schema| {
                                    let entity = cx.new(|_| {
                                        SchemaNode::new(schema, client.clone(), "main".to_string())
                                    });
                                    cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                                    entity
                                })
                                .collect();
                            this.children = ConnectionChildren::Schemas(children);
                        }
                        Err(e) => {
                            this.status =
                                ConnectionStatus::Error(format!("Không thể tải schemas: {e}"));
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
            _ => {
                let result = client.list_databases().await;

                this.update(cx, |this, cx| {
                    match result {
                        Ok(databases) => {
                            let children = databases
                                .into_iter()
                                .map(|db| {
                                    let entity = cx.new(|_| DatabaseNode::new(db, client.clone()));
                                    cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                                    entity
                                })
                                .collect();
                            this.children = ConnectionChildren::Databases(children);
                        }
                        Err(e) => {
                            this.status =
                                ConnectionStatus::Error(format!("Không thể tải databases: {e}"));
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }
}
