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

/// Đại diện cho một kết nối đến cơ sở dữ liệu, bao gồm tên, cấu hình, trạng thái và danh sách
/// các cơ sở dữ liệu.
pub struct DatabaseConnection {
    pub name: SharedString,
    pub config: DatabaseConfig,
    pub status: ConnectionStatus,
    pub databases: Vec<DatabaseNode>,
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

                    let database_nodes: Vec<DatabaseNode> =
                        databases.into_iter().map(DatabaseNode::new).collect();

                    this.update(cx, |this, cx| {
                        this.client = Some(client);
                        this.databases = database_nodes;
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
                    let database_nodes: Vec<DatabaseNode> =
                        databases.into_iter().map(DatabaseNode::new).collect();

                    this.update(cx, |this, cx| {
                        this.client = Some(client);
                        this.config.set_database(&db_name_for_async);
                        this.databases = database_nodes;
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

            let database_nodes: Vec<DatabaseNode> =
                databases.into_iter().map(DatabaseNode::new).collect();

            this.update(cx, |this, cx| {
                this.databases = database_nodes;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn load_schemas(&mut self, db_name: &str, cx: &mut Context<Self>) {
        let client = if let Some(c) = &self.client {
            c.clone()
        } else {
            return;
        };

        let databases = self.databases.clone();

        let db_to_expand = databases
            .iter()
            .find(|db| db.database.name == db_name)
            .map(|db| db.database.clone());

        if db_to_expand.is_none() {
            return;
        }

        let db_to_expand = db_to_expand.unwrap();

        cx.spawn(async move |this, cx| {
            let schemas = client.list_schemas().await.unwrap_or_default();

            let schema_nodes: Vec<SchemaNode> = schemas.into_iter().map(SchemaNode::new).collect();

            this.update(cx, |this, ctx| {
                if let Some(db) = this
                    .databases
                    .iter_mut()
                    .find(|d| d.database.name == db_to_expand.name)
                {
                    db.schemas_state = LoadState::Loaded(schema_nodes);
                }
                ctx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn load_schema_tables(&mut self, db_name: &str, schema_name: &str, cx: &mut Context<Self>) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => return,
        };
        let db_name_owned = db_name.to_string();
        let schema_name_owned = schema_name.to_string();

        cx.spawn(async move |this, cx| {
            let tables = client
                .list_tables(&schema_name_owned)
                .await
                .unwrap_or_default();

            this.update(cx, |this, ctx| {
                if let Some(db) = this
                    .databases
                    .iter_mut()
                    .find(|d| d.database.name == db_name_owned)
                {
                    if let Some(schemas) = db.schemas_state.as_loaded_mut() {
                        if let Some(schema) = schemas
                            .iter_mut()
                            .find(|s| s.schema.name == schema_name_owned)
                        {
                            schema.tables_state = LoadState::Loaded(tables);
                        }
                    }
                }
                ctx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn load_schema_views(&mut self, db_name: &str, schema_name: &str, cx: &mut Context<Self>) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => return,
        };
        let db_name_owned = db_name.to_string();
        let schema_name_owned = schema_name.to_string();

        cx.spawn(async move |this, cx| {
            let views = client
                .list_views(&schema_name_owned)
                .await
                .unwrap_or_default();

            this.update(cx, |this, ctx| {
                if let Some(db) = this
                    .databases
                    .iter_mut()
                    .find(|d| d.database.name == db_name_owned)
                {
                    if let Some(schemas) = db.schemas_state.as_loaded_mut() {
                        if let Some(schema) = schemas
                            .iter_mut()
                            .find(|s| s.schema.name == schema_name_owned)
                        {
                            schema.views_state = LoadState::Loaded(views);
                        }
                    }
                }
                ctx.notify();
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
