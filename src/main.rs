use crate::ws::handler::ws_handler;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use env_logger::Env;
use futures::{sink::SinkExt, stream::StreamExt};
use iced::widget::{button, column, text, Button, Column};
use iced::{executor, Application, Command, Element, Settings, Theme};
use log::info;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use ws::handler::AppState;
use crate::db::query_builder::{DatabaseManager, DatabaseType, DatabasePool};

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

    let mut db_manager = DatabaseManager::new();

    // Add PostgreSQL pool
    let postgres_pool = Box::new(db::postgres_custom::PostgresPool::new("your_postgres_connection_string_here").await);
    db_manager.add_pool(DatabaseType::PostgreSQL, postgres_pool);

    // Add MongoDB pool
    let mongo_pool = Box::new(db::mongodb_custom::MongoPool::new("your_mongodb_connection_string_here").await);
    db_manager.add_pool(DatabaseType::MongoDB, mongo_pool);
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