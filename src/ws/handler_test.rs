use axum::{Router, routing::get};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{AppState, ws_handler};

#[cfg(test)]
mod test {
    use super::*;

    mod test_utils {
        use super::*;
        use tokio_tungstenite::WebSocketStream;
        use tokio::net::TcpStream;
        use tokio_tungstenite::MaybeTlsStream;

        pub struct TestApp {
            pub addr: SocketAddr,
            state: Arc<AppState>,
        }

        impl TestApp {
            pub fn client_count(&self) -> usize {
                *self.state.client_count.lock().unwrap()
            }
        }

        pub async fn spawn_app() -> TestApp {
            let (tx, _) = broadcast::channel(100);
            let state = Arc::new(AppState {
                tx,
                client_count: std::sync::Mutex::new(0),
            });

            let app = Router::new()
                .route("/ws", get(ws_handler))
                .with_state(state.clone());

            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            let server_addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });

            TestApp { addr: server_addr, state }
        }

        pub struct TestClient {
            ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        }

        impl TestClient {
            pub async fn new(addr: &SocketAddr) -> Self {
                let url = format!("ws://{}/ws", addr);
                let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
                Self { ws_stream }
            }

            pub async fn send(&mut self, message: &str) {
                self.ws_stream.send(Message::Text(message.to_string())).await.unwrap();
            }

            pub async fn send_raw(&mut self, data: &[u8]) {
                self.ws_stream.send(Message::Binary(data.to_vec())).await.unwrap();
            }

            pub async fn receive(&mut self) -> String {
                let msg = self.ws_stream.next().await.unwrap().unwrap();
                msg.into_text().unwrap()
            }

            pub async fn close(mut self) {
                self.ws_stream.close(None).await.unwrap();
            }
        }
    }

    use test_utils::{spawn_app, TestClient};

    #[tokio::test]
    async fn test_websocket_connection_successful() {
        let app = spawn_app().await;
        let url = format!("ws://{}/ws", app.addr);
        let (_, response) = connect_async(url).await.expect("Failed to connect");
        assert_eq!(response.status(), 101, "WebSocket upgrade should be successful");
    }

    #[tokio::test]
    async fn test_websocket_message_broadcast() {
        let app = spawn_app().await;
        let mut client1 = TestClient::new(&app.addr).await;
        let mut client2 = TestClient::new(&app.addr).await;

        let test_message = "Hello, WebSocket!";
        client1.send(test_message).await;

        let received1 = client1.receive().await;
        let received2 = client2.receive().await;

        assert_eq!(received1, test_message, "Client 1 should receive its own message");
        assert_eq!(received2, test_message, "Client 2 should receive the broadcast message");
    }

    #[tokio::test]
    async fn test_client_count_tracking() {
        let app = spawn_app().await;

        let client1 = TestClient::new(&app.addr).await;
        assert_eq!(app.client_count(), 1, "Client count should be 1 after first connection");

        let client2 = TestClient::new(&app.addr).await;
        assert_eq!(app.client_count(), 2, "Client count should be 2 after second connection");

        drop(client1);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(app.client_count(), 1, "Client count should be 1 after disconnection");

        drop(client2);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(app.client_count(), 0, "Client count should be 0 after all disconnections");
    }

    #[tokio::test]
    async fn test_server_handles_client_disconnect_gracefully() {
        let app = spawn_app().await;
        let client = TestClient::new(&app.addr).await;

        // Abruptly close the connection
        client.close().await;

        // Give some time for the server to process the disconnection
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert_eq!(app.client_count(), 0, "Client count should be 0 after abrupt disconnection");
    }

    #[tokio::test]
    async fn test_server_handles_malformed_messages() {
        let app = spawn_app().await;
        let mut client = TestClient::new(&app.addr).await;

        // Send a malformed message (invalid UTF-8)
        let malformed_message = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
        client.send_raw(&malformed_message).await;

        // The server should not crash, and the client should still be connected
        assert_eq!(app.client_count(), 1, "Client should still be connected after malformed message");

        // Client should be able to send and receive normal messages after malformed message
        client.send("Valid message").await;
        let received = client.receive().await;
        assert_eq!(received, "Valid message", "Server should handle valid messages after receiving malformed ones");
    }
}