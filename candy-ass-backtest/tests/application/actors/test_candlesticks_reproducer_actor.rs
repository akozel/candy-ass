#[cfg(test)]
mod tests {
    use actix::Actor;
    use candy_ass_backtest::application::history_reproducer::candlesticks_reproducer_actor::CandlesticksReproducerActor;
    use candy_ass_backtest::application::history_reproducer::candlesticks_reproducer_actor::commands::ProduceCandlesticks;
    use candy_ass_backtest::application::history_reproducer::candlesticks_reproducer_actor::errors::ReproduceHistoryError;
    use candy_ass_backtest::integrations::clickhouse::ClickhouseRepositoryError;
    use candy_ass_backtest::mocks::mock_clickhouse::MockClickhouse;
    use candy_ass_core::domain::exchange_type::ExchangeType::Binance;
    use candy_ass_core::domain::symbol::Symbol;
    use candy_ass_core::domain::timeframe::Timeframe::OneDay;
    use candy_ass_core::mocks::fixtures::mock_candlesticks;
    use clickhouse::error::Error::RowNotFound;
    use futures_util::StreamExt;
    use std::sync::Arc;
    use time::{Duration, OffsetDateTime};
    use tokio_stream::wrappers::ReceiverStream;

    #[actix::test]
    async fn test_reproducer_actor() {
        let symbol = Symbol::from_pool(Binance, "BTC".into(), "USDT".into());
        let candlesticks = mock_candlesticks(symbol).await.expect("expected mock candlesticks");

        let mut clickhouse = MockClickhouse::new();
        clickhouse.expect_fetch_candlesticks_between().returning(move |_, _, _| {
            let candlesticks = candlesticks.clone();
            Box::pin(async move { Ok(candlesticks) })
        });

        // Given
        let actor = CandlesticksReproducerActor::new(2, Arc::new(clickhouse)).start();

        // When
        let command = ProduceCandlesticks {
            timeframes: vec![OneDay],
            start_date: OffsetDateTime::now_utc() - Duration::days(1),
            end_date: OffsetDateTime::now_utc(),
            step: Duration::days(1),
        };

        let candlesticks_reproducer_actor = actor.send(command.clone()).await.unwrap().unwrap();
        let err = actor.send(command.clone()).await.unwrap().unwrap_err();
        assert_eq!(err, ReproduceHistoryError::ActorIsBusy);

        let result = ReceiverStream::new(candlesticks_reproducer_actor)
            .flat_map(|(_date_time, candlesticks)| futures::stream::iter(candlesticks))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(60, result.len());
    }

    #[actix::test]
    async fn test_reproducer_actor_failed() {
        let mut clickhouse = MockClickhouse::new();
        clickhouse
            .expect_fetch_candlesticks_between()
            .returning(move |_, _, _| Box::pin(async move { Err(ClickhouseRepositoryError::UnexpectedResult(RowNotFound)) }));

        // Given
        let actor = CandlesticksReproducerActor::new(2, Arc::new(clickhouse)).start();

        // When
        let command = ProduceCandlesticks {
            timeframes: vec![OneDay],
            start_date: OffsetDateTime::now_utc() - Duration::days(1),
            end_date: OffsetDateTime::now_utc(),
            step: Duration::days(1),
        };

        let candlesticks_reproducer_actor = actor.send(command.clone()).await.unwrap().unwrap();

        let result = ReceiverStream::new(candlesticks_reproducer_actor)
            .flat_map(|(_date_time, candlesticks)| futures::stream::iter(candlesticks))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(0, result.len());
    }
}
