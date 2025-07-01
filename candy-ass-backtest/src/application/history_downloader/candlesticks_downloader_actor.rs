pub mod commands;
pub mod errors;
pub mod queries;

use crate::application::history_downloader::candlesticks_downloader_actor::Status::Ready;
use actix::{Actor, Context};
use candy_ass_core::integrations::http::binance::spot_http_client::KlinesApi;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
enum Status {
    Ready,
    Busy,
}

pub struct CandlesticksDownloaderActor {
    downstream_buffer: usize,
    concurrency: usize,
    binance_client: Arc<dyn KlinesApi + Send + Sync>,
    binance_rate_limit: usize,
    status: Status,
}

impl CandlesticksDownloaderActor {
    pub fn new(
        downstream_buffer: usize,
        concurrency: usize,
        binance_client: Arc<dyn KlinesApi + Send + Sync>,
        binance_rate_limit: usize,
    ) -> CandlesticksDownloaderActor {
        info!("Running history streaming actor with concurrency: {}", concurrency);
        CandlesticksDownloaderActor {
            downstream_buffer,
            concurrency,
            binance_client,
            binance_rate_limit,
            status: Ready,
        }
    }
}

impl Actor for CandlesticksDownloaderActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("[CandlesticksDownloaderActor] is stopped");
    }
}
