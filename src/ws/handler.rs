use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use crate::db::query_builder::DatabaseManager;

/// Shared state to keep track of the number of connected clients
pub struct AppState {
    pub(crate) tx: broadcast::Sender<String>,
    pub(crate) client_count: Mutex<usize>,
    pub db_manager: DatabaseManager,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Increment the client count
    {
        let mut client_count = state.client_count.lock().unwrap();
        *client_count += 1;
        println!("Client connected! Total clients: {}", *client_count);
    }

    // Task for sending messages to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task for receiving messages from the client
    let tx = state.tx.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received message: {}", text);
                    let _ = tx.send(text);
                }
                Ok(Message::Binary(_)) => {
                    println!("Received binary data");
                }
                Ok(Message::Ping(_)) => {
                    println!("Received ping");
                }
                Ok(Message::Pong(_)) => {
                    println!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    println!("Received close message");
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for both tasks to complete
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    // Decrement the client count
    {
        let mut client_count = state.client_count.lock().unwrap();
        *client_count -= 1;
        println!("Client disconnected! Total clients: {}", *client_count);
    }
}
