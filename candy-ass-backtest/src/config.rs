use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::Path;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Deserialize)]
pub struct ClickhouseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub clickhouse: ClickhouseConfig,
}

impl AppConfig {
    pub fn default_setup(level_filter: LevelFilter) {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_thread_names(true)
            .with_max_level(level_filter)
            .init();
    }
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        let cfg = Config::builder()
            .add_source(config::Environment::with_prefix("CANDY").separator("__"))
            .build()?;
        cfg.try_deserialize()
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        Config::builder().add_source(File::from(path.as_ref())).build()?.try_deserialize()
    }
}