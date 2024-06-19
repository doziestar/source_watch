//! This module provides configuration management for the application.
//! It loads configuration from a TOML file, prioritizes URLs if provided,
//! and initializes connections to various databases and logging systems.
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;

/// Represents errors that can occur during configuration loading.
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(err) => write!(f, "IO error: {}", err),
            ConfigError::ParseError(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError::ParseError(err)
    }
}

/// Configuration for the entire application.
#[derive(Deserialize)]
pub struct Config {
    pub databases: Databases,
    pub logs: Logs,
}

/// Contains database and logging configurations.
#[derive(Deserialize)]
pub struct Databases {
    pub mysql: Option<DatabaseConfig>,
    pub postgresql: Option<DatabaseConfig>,
    pub mongodb: Option<DatabaseConfig>,
    pub redis: Option<RedisConfig>,
}

/// Database configuration
#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

/// Redis configuration
#[derive(Deserialize)]
pub struct RedisConfig {
    pub name: String,
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
}

/// Logging configuration
#[derive(Deserialize)]
pub struct Logs {
    pub file: LogConfig,
}

/// Log file configuration
#[derive(Deserialize)]
pub struct LogConfig {
    pub name: String,
    pub path: String,
    pub format: String,
    pub encoding: String,
    pub interval: u32,
}

/// Loads configuration from a TOML file.
impl Config {
    pub fn from_file(file: &str) -> Result<Config, ConfigError> {
        let config_str = fs::read_to_string(file)?;
        let config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
