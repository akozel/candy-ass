#[cfg(test)]
mod smoke_test {
    use candy_ass_backtest::application::{history_downloader, history_reproducer};
    use candy_ass_backtest::config::AppConfig;
    use candy_ass_backtest::mocks::mock_docker_clickhouse::setup_clickhouse_container;
    use candy_ass_core::domain::symbol::Symbol;
    use candy_ass_core::domain::timeframe::Timeframe::OneHour;
    use futures_util::StreamExt;
    use std::sync::Arc;
    use time::{OffsetDateTime, UtcOffset};

    fn yesterday() -> OffsetDateTime {
        let now = OffsetDateTime::now_utc();
        let today = now.date();
        let yesterday = today.previous_day().unwrap();
        yesterday.with_time(time::Time::MIDNIGHT).assume_offset(UtcOffset::UTC)
    }

    #[actix::test]
    async fn smoke_test() {
        let config = AppConfig::from_file("tests/default.yaml").unwrap();
        let _container = setup_clickhouse_container(&config.clickhouse).await;

        // Setup downloader
        let downloader_app = history_downloader::Application::new(10, 4, config);

        let now = OffsetDateTime::now_utc();
        let start_date = yesterday();
        let filter: Arc<dyn Fn(&Arc<Symbol>) -> bool + Send + Sync> =
            Arc::new(|symbol| symbol.quote_asset == "USDT" && (symbol.base_asset == "BTC" || symbol.base_asset == "ETH"));

        // Run downloader
        downloader_app.start_pipeline(OneHour, start_date, filter).await;
        downloader_app.run_optimization().await;

        // Setup reproducer
        let config = AppConfig::from_file("tests/default.yaml").unwrap();
        let reproducer_app = history_reproducer::Application::new(2, config);

        // Run reproducer
        let stream = reproducer_app
            .start_pipeline(vec![OneHour], start_date, now)
            .await
            .flat_map(|(_date_time, candlesticks)| futures::stream::iter(candlesticks))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(start_date, stream[0].open_time);
        assert_eq!(start_date, stream[1].open_time);

        assert_eq!("BTC", stream[0].symbol.base_asset);
        assert_eq!("ETH", stream[1].symbol.base_asset);

        assert!(48usize <= stream.len());
        assert!(96usize >= stream.len());
    }
}
