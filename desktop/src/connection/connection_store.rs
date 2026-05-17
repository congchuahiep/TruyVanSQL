use crate::connection::database_connection::DatabaseConnection;
use engine::{DatabaseConfig, SqlClient};
use gpui::*;

pub struct ConnectionStore {
    connections: Vec<Entity<DatabaseConnection>>,
    active_index: Option<usize>,
}

impl ConnectionStore {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            connections: Vec::new(),
            active_index: None,
        }
    }

    pub fn add_connection(
        &mut self,
        name: SharedString,
        config: DatabaseConfig,
        cx: &mut Context<Self>,
    ) {
        let connection = cx.new(|cx| DatabaseConnection::new(name, config, cx));

        connection.update(cx, |c, cx| c.connect(cx));

        cx.observe(&connection, |_, _, cx| cx.notify()).detach();

        self.connections.push(connection);

        if self.active_index.is_none() {
            self.active_index = Some(0);
        }

        cx.notify();
    }

    pub fn connections(&self) -> &[Entity<DatabaseConnection>] {
        &self.connections
    }

    pub fn active_connection(&self) -> Option<Entity<DatabaseConnection>> {
        self.active_index
            .and_then(|i| self.connections.get(i).cloned())
    }

    pub fn set_active(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.connections.len() {
            self.active_index = Some(index);
            cx.notify();
        }
    }

    pub fn remove_connection(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.connections.len() {
            self.connections.remove(index);
            if let Some(active) = self.active_index {
                if active >= self.connections.len() {
                    self.active_index = self.connections.len().checked_sub(1);
                }
            }
            cx.notify();
        }
    }
}
