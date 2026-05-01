use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub clickhouse: ClickHouseConfig,
    pub nats: NatsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub env: String,
    pub host: String,
    pub port: u16,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub pass: String,
    pub db_name: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        // URL-encode password to handle special characters
        let encoded_pass = urlencoding::encode(&self.pass);
        format!(
            "postgres://{}:{}@{}:5432/{}",
            self.user, encoded_pass, self.host, self.db_name
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClickHouseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl ClickHouseConfig {
    pub fn connection_string(&self) -> String {
        format!("clickhouse://{}:{}/{}", self.host, self.port, self.database)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NatsConfig {
    pub host: String,
    pub port: u16,
}

impl NatsConfig {
    pub fn connection_string(&self) -> String {
        format!("nats://{}:{}", self.host, self.port)
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config"))
            // Environment variable overrides
            .add_source(config::Environment::with_prefix("GRAVITY").separator("__"))
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize()?;

        // Load RSA keys from files if paths are provided
        if !app_config.server.jwt_private_key.is_empty()
            && app_config.server.jwt_private_key.ends_with(".pem")
        {
            app_config.server.jwt_private_key =
                fs::read_to_string(&app_config.server.jwt_private_key).map_err(|e| {
                    ConfigError::Message(format!("Failed to read JWT private key: {}", e))
                })?;
        }
        if !app_config.server.jwt_public_key.is_empty()
            && app_config.server.jwt_public_key.ends_with(".pem")
        {
            app_config.server.jwt_public_key =
                fs::read_to_string(&app_config.server.jwt_public_key).map_err(|e| {
                    ConfigError::Message(format!("Failed to read JWT public key: {}", e))
                })?;
        }

        // Validate JWT secret in dev mode
        if app_config.server.env == "prod" {
            if app_config.server.jwt_private_key.contains("dev_secret")
                || app_config.server.jwt_public_key.contains("PUBLIC KEY")
            {
                return Err(ConfigError::Message(
                    "Production environment must use real RSA keys, not dev defaults".into(),
                ));
            }
        }

        Ok(app_config)
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut encoded = String::new();
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}
