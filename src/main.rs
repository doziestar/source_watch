use crate::ws::handler::ws_handler;
use axum::{
    routing::get,
    Router,
};
use log::{info, error};
use std::sync::{Arc};
use ws::handler::AppState;

mod config;
mod db;
mod logging;
mod models;
mod utils;
mod ws;

#[tokio::main]
async fn main() {
    logging::init();
    info!("Starting the SourceWatch application...");

    let state = ws::state::init_app_state().await;
    let app = create_app(state);

    let addr = "0.0.0.0:6262";
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(Arc::from(state))
}