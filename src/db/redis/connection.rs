use redis::Commands;
use std::env;

pub fn establish_connection() -> redis::Connection {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    client.get_connection().expect("Failed to connect to Redis")
}
