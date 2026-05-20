use engine::{SchemaBrief, SqlClient, TableBrief};
use gpui::*;

use crate::shared::LoadState;

/// Đại diện cho một node schema trong cây hiển thị.
/// Chứa thông tin về schema và danh sách các bảng, view bên trong.
/// Mỗi SchemaNode là một Entity tự quản lý việc load dữ liệu.
pub struct SchemaNode {
    pub schema: SchemaBrief,
    pub tables_state: LoadState<Vec<TableBrief>>,
    pub views_state: LoadState<Vec<TableBrief>>,
    client: SqlClient,
    db_name: String,
}

impl std::fmt::Debug for SchemaNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaNode")
            .field("schema", &self.schema)
            .field("tables_state", &self.tables_state)
            .field("views_state", &self.views_state)
            .field("db_name", &self.db_name)
            .finish()
    }
}

impl SchemaNode {
    /// Tạo SchemaNode mới với engine types.
    pub fn new(schema: SchemaBrief, client: SqlClient, db_name: String) -> Self {
        Self {
            schema,
            tables_state: LoadState::Idle,
            views_state: LoadState::Idle,
            client,
            db_name,
        }
    }

    /// Kiểm tra xem đang ở trạng thái idle (chưa load bao giờ)
    pub fn is_idle(&self) -> bool {
        self.tables_state.is_idle() && self.views_state.is_idle()
    }

    /// Load danh sách tables cho schema này.
    pub fn load_tables(&mut self, cx: &mut Context<Self>) {
        if !self.tables_state.is_idle() {
            return;
        }

        self.tables_state = LoadState::Loading;
        cx.notify();

        let client = self.client.clone();
        let schema_name = self.schema.name.clone();

        cx.spawn(async move |this, cx| {
            let tables = client.list_tables(&schema_name).await.unwrap_or_default();
            this.update(cx, |this, cx| {
                this.tables_state = LoadState::Loaded(tables);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Load danh sách views cho schema này.
    pub fn load_views(&mut self, cx: &mut Context<Self>) {
        if !self.views_state.is_idle() {
            return;
        }

        self.views_state = LoadState::Loading;
        cx.notify();

        let client = self.client.clone();
        let schema_name = self.schema.name.clone();

        cx.spawn(async move |this, cx| {
            let views = client.list_views(&schema_name).await.unwrap_or_default();
            this.update(cx, |this, cx| {
                this.views_state = LoadState::Loaded(views);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
