use crate::config::ClickhouseConfig;
use ::clickhouse::{Client, Compression};
use std::sync::Arc;

pub mod clickhouse;

pub fn clickhouse_client(config: ClickhouseConfig) -> Arc<Client> {
    let client = Client::default()
        .with_compression(Compression::None)
        .with_url(format!("{}:{}", config.host, config.port))
        .with_user(config.username)
        .with_password(config.password);
    Arc::new(client)
}
