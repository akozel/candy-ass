use crate::config::ClickhouseConfig;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub async fn setup_clickhouse_container(config: &ClickhouseConfig) -> ContainerAsync<GenericImage> {
    GenericImage::new("clickhouse/clickhouse-server", "24")
        .with_wait_for(WaitFor::message_on_stderr("Logging trace to"))
        .with_wait_for(WaitFor::seconds(2))
        .with_mapped_port(config.port, 8123.tcp())
        .with_env_var("CLICKHOUSE_USER", &config.username)
        .with_env_var("CLICKHOUSE_PASSWORD", &config.password)
        .start()
        .await
        .expect("Failed to start clickhouse/clickhouse-server")
}
