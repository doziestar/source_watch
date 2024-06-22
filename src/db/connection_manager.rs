use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::db::Database;

pub struct ConnectionManager {
    connections: Mutex<HashMap<String, Box<dyn Database>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_connection(&self, name: &str, db: Box<dyn Database>) {
        let mut connections = self.connections.lock().await;
        connections.insert(name.to_string(), db);
    }

    pub async fn get_connection(&self, name: &str) -> Option<Box<dyn Database>> {
        let connections = self.connections.lock().await;
        connections.get(name).cloned()
    }

    pub async fn remove_connection(&self, name: &str) {
        let mut connections = self.connections.lock().await;
        connections.remove(name);
    }
}