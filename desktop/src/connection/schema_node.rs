use engine::{SchemaBrief, TableBrief};

use crate::shared::LoadState;

/// Đại diện cho một node schema trong cây hiển thị.
/// Chứa thông tin về schema và danh sách các bảng, view bên trong.
#[derive(Debug, Clone)]
pub struct SchemaNode {
    pub schema: SchemaBrief,
    pub tables_state: LoadState<Vec<TableBrief>>,
    pub views_state: LoadState<Vec<TableBrief>>,
}

impl SchemaNode {
    pub fn new(schema: SchemaBrief) -> Self {
        Self {
            schema,
            tables_state: LoadState::Idle,
            views_state: LoadState::Idle,
        }
    }
}
