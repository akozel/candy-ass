#[cfg(test)]
mod integration_tests {
    use candy_ass_backtest::config::{AppConfig, ClickhouseConfig};
    use candy_ass_backtest::integrations::clickhouse::candlesticks_repository::{CandlesticksReadService, CandlesticksRepository, CandlesticksWriteService};
    use candy_ass_backtest::integrations::clickhouse_client;
    use candy_ass_backtest::mocks::mock_docker_clickhouse::setup_clickhouse_container;
    use candy_ass_core::domain::timeframe::Timeframe::OneDay;
    use candy_ass_core::mocks::fixtures::BTC_USDT_CANDLESTICK;
    use testcontainers::{ContainerAsync, GenericImage};
    use time::format_description::well_known::Rfc3339;
    use time::{Duration, OffsetDateTime};

    pub async fn setup_repository(config: ClickhouseConfig) -> (ContainerAsync<GenericImage>, CandlesticksRepository) {
        let container = setup_clickhouse_container(&config).await;
        let clinet = clickhouse_client(config);
        (container, CandlesticksRepository::new(clinet))
    }

    #[tokio::test]
    async fn integration_scenario() {
        // setup
        let config = AppConfig::from_file("tests/default.yaml").unwrap().clickhouse;
        let (_container, repository) = setup_repository(config).await;
        let start_date = OffsetDateTime::parse("2024-01-01T00:00:00Z", &Rfc3339).unwrap();

        // flow
        let _ = repository.init().await;
        let _ = repository.bulk_insert_candlesticks(vec![vec![BTC_USDT_CANDLESTICK.clone()]]).await;

        let result = repository
            .fetch_candlesticks_between(vec![OneDay], start_date, start_date + Duration::days(1))
            .await
            .unwrap();

        assert_eq!(1, result.len());
        assert_eq!(100_000.0, result[0].open_price);
        assert_eq!(101_000.0, result[0].close_price);
    }
}
