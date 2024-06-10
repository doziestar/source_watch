use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub mongo_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_file(file: &str) -> Config {
        let config_str = fs::read_to_string(file).expect("Failed to read config file");
        toml::from_str(&config_str).expect("Failed to parse config")
    }
}
