#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::{Config, ConfigError, DatabaseConfig};
    use std::fs::File;
    use std::io::Write;

    fn create_test_config(content: &str, file_path: &str) {
        let mut file = File::create(file_path).expect("Failed to create test config file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to test config file");
    }

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
            [databases.mysql]
            name = "MySQL Database"
            url = "mysql://user:password@localhost:3306/test"

            [databases.postgresql]
            name = "PostgreSQL Database"
            url = "postgres://user:password@localhost:5432/test"

            [logs.file]
            name = "Log File"
            path = "/var/log/app.log"
            format = "json"
            encoding = "utf-8"
            interval = 1
        "#;
        create_test_config(config_content, "test_config.toml");

        let config = Config::from_file("test_config.toml").expect("Failed to load config");
        assert_eq!(config.databases.mysql.unwrap().name, "MySQL Database");
        assert_eq!(
            config.databases.postgresql.unwrap().name,
            "PostgreSQL Database"
        );
        assert_eq!(config.logs.file.name, "Log File");
    }

    #[test]
    fn test_load_invalid_config() {
        let config_content = r#"
            [databases.mysql]
            name = "MySQL Database"
            url = "mysql://user:password@localhost:3306/test"

            [logs.file]
            name = "Log File"
            path = "/var/log/app.log"
            format = "json"
            encoding = "utf-8"
            interval = 1
        "#;
        create_test_config(config_content, "invalid_config.toml");

        match Config::from_file("invalid_config.toml") {
            Ok(_) => panic!("Expected to fail on invalid config"),
            Err(e) => assert!(matches!(e, ConfigError::ParseError(_))),
        }
    }

    #[test]
    fn test_initialize_mysql_with_url() {
        let db_config = DatabaseConfig {
            name: "MySQL Database".to_string(),
            url: Some("mysql://user:password@localhost:3306/test".to_string()),
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
        };

        // assert!(initialize_mysql(db_config).is_ok());
    }

    #[test]
    fn test_initialize_postgresql_with_url() {
        let db_config = DatabaseConfig {
            name: "PostgreSQL Database".to_string(),
            url: Some("postgres://user:password@localhost:5432/test".to_string()),
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
        };

        // assert!(initialize_postgresql(db_config).is_ok());
    }
}
