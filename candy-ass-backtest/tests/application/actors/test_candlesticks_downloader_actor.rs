#[cfg(test)]
mod tests {
    use actix::Actor;
    use candy_ass_backtest::application::history_downloader::candlesticks_downloader_actor::CandlesticksDownloaderActor;
    use candy_ass_backtest::application::history_downloader::candlesticks_downloader_actor::commands::download_candlesticks::DownloadCandlesticks;
    use candy_ass_backtest::application::history_downloader::candlesticks_downloader_actor::commands::shutdown::Command::Shutdown;
    use candy_ass_backtest::application::history_downloader::candlesticks_downloader_actor::errors::DownloadHistoryError;
    use candy_ass_core::domain::candlestick::Candlestick;
    use candy_ass_core::domain::exchange_type::ExchangeType::Binance;
    use candy_ass_core::domain::symbol::Symbol;
    use candy_ass_core::domain::timeframe::Timeframe::ThreeMinutes;
    use candy_ass_core::integrations::binance_spot_client;
    use candy_ass_core::integrations::http::binance::BINANCE_RATE_LIMIT;
    use reqwest::Client;
    use std::sync::Arc;
    use time::{Duration, OffsetDateTime};
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::ReceiverStream;

    #[actix::test]
    async fn test_history_streaming_actor() {
        // Given
        let binance_client = binance_spot_client(Client::new());

        let history_streaming_actor = CandlesticksDownloaderActor::new(10, 3, binance_client, BINANCE_RATE_LIMIT).start();
        let btc_usdt = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        let start_date = OffsetDateTime::now_utc() - Duration::days(3);

        let msg = DownloadCandlesticks {
            symbols: Arc::new(vec![btc_usdt]),
            timeframe: ThreeMinutes,
            start_date,
            filter: Arc::new(|symbol| symbol.quote_asset == "USDT"),
        };

        // When
        let receiver = history_streaming_actor.send(msg.clone()).await.unwrap().unwrap();
        let err = history_streaming_actor.send(msg.clone()).await.unwrap().unwrap_err();
        assert_eq!(err, DownloadHistoryError::ActorIsBusy);

        let result = ReceiverStream::new(receiver).collect::<Vec<Vec<Candlestick>>>().await;

        assert_eq!(2, result.len());
        history_streaming_actor.send(Shutdown).await.unwrap();
    }
}
