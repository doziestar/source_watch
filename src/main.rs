use crate::ws::handler::ws_handler;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use env_logger::Env;
use futures::{sink::SinkExt, stream::StreamExt};
use log::{info, error};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use ws::handler::AppState;
use crate::db::query_builder::DatabaseManager;
use crate::db::db_manager;

mod config;
mod db;
mod logging;
mod models;
mod utils;
mod ws;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the SourceWatch application...");

    let (tx, _) = broadcast::channel(100);

    let db_manager = match db_manager::initialize_db_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize database manager: {}", e);
            return;
        }
    };

    // Create the shared state
    let state = Arc::new(AppState {
        tx,
        client_count: Mutex::new(0),
        db_manager,
    });

    // Build app with a route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    // Start the server
    let addr = "0.0.0.0:6262";
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}