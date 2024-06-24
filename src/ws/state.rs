use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use crate::db::db_manager::{DatabaseManager, initialize_db_manager};

/// The `AppState` struct holds the application state.
/// It contains a broadcast channel sender, a mutex-wrapped client count, and a `DatabaseManager`.
/// The `tx` field is a broadcast channel sender that is used to send messages to all connected clients.
/// The `client_count` field is a mutex-wrapped integer that keeps track of the number of connected clients.
/// The `db_manager` field is a `DatabaseManager` that is used to interact with the database.
/// The `AppState` struct is used to share state between different parts of the application.
/// Using `Arc<AppState>` allows multiple parts of the application to have read-only access to the state.
/// Usage:
/// ```rust
/// use std::sync::Arc;
/// use tokio::sync::broadcast;
/// use tokio::sync::Mutex;
/// use crate::db::db_manager::DatabaseManager;
/// use crate::ws::state::AppState;
///
/// let (tx, _) = broadcast::channel(100);
/// let db_manager = DatabaseManager::new();
/// let client_count = Mutex::new(0);
///
/// let app_state = Arc::new(AppState {
///    tx,
///   client_count,
///  db_manager,
/// });
///
/// assert_eq!(app_state.client_count.lock().unwrap(), 0);
/// ```
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub client_count: Mutex<usize>,
    pub db_manager: DatabaseManager,
}

/// The `init_app_state` function initializes the application state.
/// It creates a broadcast channel sender, initializes the database manager, and returns an `Arc<AppState>`.
/// The `init_app_state` function is used to set up the application state before starting the server.
pub async fn init_app_state() -> Arc<AppState> {
    let (tx, _) = broadcast::channel(100);
    let db_manager = initialize_db_manager().await.expect("Failed to initialize database manager");

    Arc::new(AppState {
        tx,
        client_count: Mutex::new(0),
        db_manager,
    })
}