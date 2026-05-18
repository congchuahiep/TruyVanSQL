use engine::DatabaseBrief;

use crate::shared::LoadState;

use super::SchemaNode;

/// Đại diện cho một node cơ sở dữ liệu trong cây hiển thị.
/// Chứa thông tin về database và danh sách các schema bên trong.
#[derive(Debug, Clone)]
pub struct DatabaseNode {
    pub database: DatabaseBrief,
    pub schemas_state: LoadState<Vec<SchemaNode>>,
}

impl DatabaseNode {
    pub fn new(database: DatabaseBrief) -> Self {
        Self {
            database,
            schemas_state: LoadState::Idle,
        }
    }
}
