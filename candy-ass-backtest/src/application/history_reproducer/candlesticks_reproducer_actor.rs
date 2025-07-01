pub mod commands;
pub mod errors;

use crate::integrations::clickhouse::candlesticks_repository::CandlesticksReadService;
use actix::{Actor, Context};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
enum Status {
    Ready,
    Busy,
}

pub struct CandlesticksReproducerActor {
    prefetch_buffer: usize,
    candlesticks_read_service: Arc<dyn CandlesticksReadService + Send + Sync>,
    status: Status,
}

impl CandlesticksReproducerActor {
    pub fn new(prefetch_buffer: usize, candlesticks_read_service: Arc<dyn CandlesticksReadService + Send + Sync>) -> CandlesticksReproducerActor {
        CandlesticksReproducerActor {
            prefetch_buffer,
            candlesticks_read_service,
            status: Status::Ready,
        }
    }
}

impl Actor for CandlesticksReproducerActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("[CandlesticksReproducerActor] is stopped");
    }
}
