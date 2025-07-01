#[cfg(test)]
mod tests {
    use actix::Actor;
    use candy_ass_core::application::actors::symbols_fetcher_actor::RefreshPolicy::Periodic;
    use candy_ass_core::application::actors::symbols_fetcher_actor::commands::Command::Shutdown;
    use candy_ass_core::application::actors::symbols_fetcher_actor::commands::RefreshAndGet;
    use candy_ass_core::application::actors::symbols_fetcher_actor::errors::FailedToFetchSymbolsError;
    use candy_ass_core::application::actors::symbols_fetcher_actor::{GetReceiver, RefreshPolicy, SymbolsFetcherActor};
    use candy_ass_core::domain::symbol::Symbols;
    use candy_ass_core::mocks::mock_binance_spot::broken::BROKEN_BINANCE_SPOT_CLIENT;
    use candy_ass_core::mocks::mock_binance_spot::default::DEFAULT_BINANCE_SPOT_CLIENT;
    use futures_util::StreamExt;
    use futures_util::future::ready;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_stream::wrappers::WatchStream;

    #[actix::test]
    async fn test_refresh_triggers_logic() {
        // Given
        let mock_binance_client = DEFAULT_BINANCE_SPOT_CLIENT.clone();
        let symbols_fetcher_actor = SymbolsFetcherActor::new(RefreshPolicy::Lazy, mock_binance_client.clone()).start();

        // When
        let receiver = symbols_fetcher_actor.send(GetReceiver).await.unwrap();
        let state = receiver.borrow().clone();
        assert_eq!(None, state);

        let _ = symbols_fetcher_actor.send(RefreshAndGet).await;

        // Then
        let state = receiver.borrow().clone().unwrap();

        symbols_fetcher_actor.send(Shutdown).await.unwrap();
        assert_eq!(2, state.len());
    }

    #[actix::test]
    async fn test_refresh_triggers_one_shot() {
        // Given
        let mock_binance_client = DEFAULT_BINANCE_SPOT_CLIENT.clone();
        let symbols_fetcher_actor = SymbolsFetcherActor::new(RefreshPolicy::OneShot, mock_binance_client.clone()).start();

        // When
        let receiver = symbols_fetcher_actor.send(GetReceiver).await.unwrap();

        // Then
        let result = WatchStream::new(receiver).filter_map(ready).next().await;
        let state = result.unwrap();

        symbols_fetcher_actor.send(Shutdown).await.unwrap();
        assert_eq!(2, state.len());
    }

    #[actix::test]
    async fn test_refresh_triggers_periodic() {
        // Given
        let mock_binance_client = DEFAULT_BINANCE_SPOT_CLIENT.clone();
        let symbols_fetcher_actor = SymbolsFetcherActor::new(Periodic(Duration::from_millis(20)), mock_binance_client.clone()).start();

        // When
        let receiver = symbols_fetcher_actor.send(GetReceiver).await.unwrap();

        // Then
        let result: Vec<Arc<Symbols>> = WatchStream::new(receiver).filter_map(ready).take(2).collect().await;

        let result0 = result.first().expect("result[0] missing").as_ref();
        let result1 = result.get(1).expect("result[1] missing").as_ref();

        symbols_fetcher_actor.send(Shutdown).await.unwrap();
        assert_eq!(2, result0.len());
        assert_eq!(2, result1.len());
    }

    #[actix::test]
    async fn test_bad_http_response() {
        // Given
        let binance_client = BROKEN_BINANCE_SPOT_CLIENT.clone();
        let symbols_fetcher_actor = SymbolsFetcherActor::new(RefreshPolicy::Lazy, binance_client.clone()).start();

        // When
        let receiver = symbols_fetcher_actor.send(GetReceiver).await.unwrap();
        let state = receiver.borrow().clone();
        assert_eq!(None, state);

        let err = symbols_fetcher_actor.send(RefreshAndGet).await.unwrap().unwrap_err();

        // Then
        assert!(matches!(err, FailedToFetchSymbolsError::Transport(_)));
    }
}
