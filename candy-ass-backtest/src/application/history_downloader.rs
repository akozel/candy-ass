use crate::application::history_downloader::candlesticks_downloader_actor::CandlesticksDownloaderActor;
use crate::application::history_downloader::candlesticks_downloader_actor::commands::download_candlesticks::DownloadCandlesticks;
use crate::config::AppConfig;
use crate::integrations::clickhouse::ClickhouseRepositoryError;
use crate::integrations::clickhouse::candlesticks_repository::{CandlesticksRepository, CandlesticksWriteService};
use crate::integrations::clickhouse_client;
use actix::{Actor, Addr};
use candy_ass_core::application::actors::symbols_fetcher_actor;
use candy_ass_core::application::actors::symbols_fetcher_actor::RefreshPolicy::OneShot;
use candy_ass_core::application::actors::symbols_fetcher_actor::{GetReceiver, SymbolsFetcherActor};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::symbol::{SymbolFilterFn, Symbols};
use candy_ass_core::domain::timeframe::Timeframe;
use candy_ass_core::integrations::binance_spot_client;
use candy_ass_core::integrations::http::HttpResponseError;
use candy_ass_core::integrations::http::binance::BINANCE_RATE_LIMIT;
use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryFutureExt, TryStreamExt};
use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::watch::error::SendError;
use tokio_stream::wrappers::{ReceiverStream, WatchStream};
use tracing::error;

pub mod candlesticks_downloader_actor;

pub struct Application {
    candlesticks_repository: Arc<CandlesticksRepository>,
    symbols_fetcher_actor: Addr<SymbolsFetcherActor>,
    candlesticks_downloader_actor: Addr<CandlesticksDownloaderActor>,
}

impl Application {
    pub fn new(downstream_buffer: usize, concurrency: usize, app_config: AppConfig) -> Self {
        // infrastructure
        let http_client = Client::new();
        let binance = binance_spot_client(http_client.clone());
        let clickhouse = clickhouse_client(app_config.clickhouse);
        let candlesticks_repository = Arc::new(CandlesticksRepository::new(clickhouse));

        // actors
        let symbols_fetcher_actor = SymbolsFetcherActor::new(OneShot, binance.clone());
        let history_streaming_actor = CandlesticksDownloaderActor::new(downstream_buffer, concurrency, binance.clone(), BINANCE_RATE_LIMIT);

        Application {
            candlesticks_repository,
            symbols_fetcher_actor: symbols_fetcher_actor.start(),
            candlesticks_downloader_actor: history_streaming_actor.start(),
        }
    }

    pub async fn start_pipeline(&self, timeframe: Timeframe, start_date: OffsetDateTime, filter: SymbolFilterFn) {
        let candlesticks_repository = self.candlesticks_repository.clone();
        let symbols_fetcher_actor = self.symbols_fetcher_actor.clone();
        let candlesticks_downloader_actor = self.candlesticks_downloader_actor.clone();

        let _ = tokio_stream::once(true)
            .then(|_| Self::init_candlestick_repository(candlesticks_repository.clone()))
            .then(|_| Self::watch_binance_symbols(symbols_fetcher_actor.clone()))
            .flatten()
            .take(1)
            .map(|symbols| Self::download_candlesticks_command(timeframe.clone(), start_date, symbols, filter.clone()))
            .then(|command| Self::download_candlesticks_into_stream(command, candlesticks_downloader_actor.clone()))
            .flat_map_unordered(8, |candlesticks| candlesticks)
            .chunks(8)
            .then(|chunk| Self::persist_candlesticks(chunk, candlesticks_repository.clone()))
            .inspect_err(|err| error!("Error during main pipeline: {}", err))
            .for_each(|_| async {})
            .await;

        let _ = symbols_fetcher_actor.send(symbols_fetcher_actor::commands::Command::Shutdown).await;

        let _ = candlesticks_downloader_actor
            .send(candlesticks_downloader_actor::commands::shutdown::Command::Shutdown)
            .await;
    }

    pub async fn run_optimization(&self) {
        let _ = self
            .candlesticks_repository
            .clone()
            .run_optimization()
            .inspect_err(|err| error!("Error during clickhouse optimization: {}", err))
            .await;
    }

    async fn init_candlestick_repository(candlesticks_repository: Arc<CandlesticksRepository>) -> Result<(), ClickhouseRepositoryError> {
        candlesticks_repository.init().await
    }

    async fn watch_binance_symbols(symbols_fetcher_actor: Addr<SymbolsFetcherActor>) -> BoxStream<'static, Arc<Symbols>> {
        async {
            let receiver = symbols_fetcher_actor.send(GetReceiver).await.expect("Failed to get symbols receiver");

            WatchStream::new(receiver).filter_map(|item| async move { item })
        }
        .await
        .boxed()
    }

    async fn download_candlesticks_into_stream(
        download: DownloadCandlesticks,
        candlesticks_downloader_actor: Addr<CandlesticksDownloaderActor>,
    ) -> BoxStream<'static, Vec<Candlestick>> {
        async {
            let candlestick_receiver = candlesticks_downloader_actor
                .send(download)
                .await
                .expect("Failed send message into [CandlesticksDownloaderActor]")
                .expect("Failed to get candlestick receiver from [CandlesticksDownloaderActor]");

            ReceiverStream::new(candlestick_receiver)
        }
        .await
        .boxed()
    }

    async fn persist_candlesticks(chunk: Vec<Vec<Candlestick>>, candlesticks_repository: Arc<CandlesticksRepository>) -> Result<(), ClickhouseRepositoryError> {
        candlesticks_repository.bulk_insert_candlesticks(chunk).await
    }

    fn download_candlesticks_command(timeframe: Timeframe, start_date: OffsetDateTime, symbols: Arc<Symbols>, filter: SymbolFilterFn) -> DownloadCandlesticks {
        DownloadCandlesticks {
            symbols,
            timeframe: timeframe.clone(),
            start_date,
            filter: filter.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum HistoryGrabberError<T> {
    #[error("HttpResponseError {source}")]
    UnexpectedHttpResponse {
        #[from]
        #[source]
        source: HttpResponseError,
    },

    #[error("Failed to send message: {0}")]
    UnexpectedSendResult(#[from] SendError<T>),

    #[error("Unexpected error {0}")]
    Unexpected(String),
}
