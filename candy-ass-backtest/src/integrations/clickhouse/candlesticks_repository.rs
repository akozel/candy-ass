pub mod read_service;
pub mod write_service;

use crate::integrations::clickhouse::ClickhouseRepositoryError;
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::timeframe::Timeframe;
use clickhouse::Client;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct CandlesticksRepository {
    client: Arc<Client>,
}

impl CandlesticksRepository {
    pub fn new(client: Arc<Client>) -> CandlesticksRepository {
        CandlesticksRepository { client }
    }
}

/// Services
pub trait CandlesticksWriteService {
    fn init(&self) -> BoxFuture<Result<(), ClickhouseRepositoryError>>;
    fn bulk_insert_candlesticks(&self, chunk: Vec<Vec<Candlestick>>) -> BoxFuture<Result<(), ClickhouseRepositoryError>>;
    fn run_optimization(&self) -> BoxFuture<Result<(), ClickhouseRepositoryError>>;
}

pub trait CandlesticksReadService {
    fn fetch_candlesticks_between(
        &self,
        timeframe: Vec<Timeframe>,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> BoxFuture<Result<Vec<Candlestick>, ClickhouseRepositoryError>>;
}
