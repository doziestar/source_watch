use mongodb::{Client, options::ClientOptions};
use std::env;

pub async fn establish_connection() -> Client {
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set");
    let client_options = ClientOptions::parse(&mongo_url).await.expect("Failed to parse options");
    Client::with_options(client_options).expect("Failed to initialize client")
}