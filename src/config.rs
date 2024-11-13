use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_storage_path")]
    pub storage_path: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}


fn default_bind_address() -> String {
    "127.0.0.1:8080".to_string()
}

fn default_storage_path() -> String {
    "./storage".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_cache_ttl() -> u64 {
    3600
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        
        dotenvy::dotenv().ok();

        
        let config = Config::builder()
            .add_source(Environment::with_prefix("WAREHOUSE"))
            .build()?;

        
        Ok(config.try_deserialize()?)
    }
}