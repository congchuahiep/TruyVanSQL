use engine::{DatabaseBrief, DatabaseConfig, SqlClient};
use gpui::*;

use crate::{connection::SchemaNode, shared::LoadState};

/// Đại diện cho một node cơ sở dữ liệu trong cây hiển thị.
/// Chứa thông tin về database và danh sách các schema bên trong.
/// Mỗi DatabaseNode là một Entity tự quản lý việc load schemas.
pub struct DatabaseNode {
    pub database: DatabaseBrief,
    pub schemas_state: LoadState<Vec<Entity<SchemaNode>>>,
    client: SqlClient,
}

impl std::fmt::Debug for DatabaseNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseNode")
            .field("database", &self.database)
            .field("schemas_state", &self.schemas_state)
            .finish()
    }
}

impl DatabaseNode {
    /// Tạo DatabaseNode mới với engine types.
    pub fn new(database: DatabaseBrief, client: SqlClient) -> Self {
        Self {
            database,
            schemas_state: LoadState::Idle,
            client,
        }
    }

    /// Kiểm tra xem đang ở trạng thái idle (chưa load bao giờ)
    pub fn is_idle(&self) -> bool {
        self.schemas_state.is_idle()
    }

    /// Load danh sách schemas cho database này.
    pub fn load_schemas(&mut self, cx: &mut Context<Self>) {
        if !self.schemas_state.is_idle() {
            return;
        }

        self.schemas_state = LoadState::Loading;
        cx.notify();

        let client = self.client.clone();
        let db_name = self.database.name.clone();

        cx.spawn(async move |this, cx| {
            let schemas = client.list_schemas().await.unwrap_or_default();

            this.update(cx, |this, cx| {
                let schema_entities: Vec<Entity<SchemaNode>> = schemas
                    .into_iter()
                    .map(|schema| {
                        let entity =
                            cx.new(|_| SchemaNode::new(schema, client.clone(), db_name.clone()));
                        cx.observe(&entity, |_, _, cx| cx.notify()).detach();
                        entity
                    })
                    .collect();

                this.schemas_state = LoadState::Loaded(schema_entities);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
