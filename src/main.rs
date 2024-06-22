use env_logger::Env;
use iced::widget::{button, column, text, Button, Column};
use iced::{executor, Application, Command, Element, Settings, Theme};
use log::info;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use ws::handler::AppState;
use crate::ws::handler::ws_handler;

mod config;
mod db;
mod logging;
mod models;
mod utils;
mod ws;


/// Main function to start the SourceWatch application
/// It initializes the logger and starts the GUI
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the SourceWatch application...");

    // Create a channel to broadcast messages to all connected clients
    let (tx, _) = broadcast::channel(100);

    // Create the shared state
    let state = Arc::new(AppState {
        tx,
        client_count: Mutex::new(0),
    });

    // Build app with a route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    // Start the server
    let addr = "0.0.0.0:6262";
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
