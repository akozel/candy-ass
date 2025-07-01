use crate::application::history_reproducer::candlesticks_reproducer_actor::CandlesticksReproducerActor;
use crate::application::history_reproducer::candlesticks_reproducer_actor::commands::ProduceCandlesticks;
use crate::config::AppConfig;
use crate::integrations::clickhouse::candlesticks_repository::CandlesticksRepository;
use crate::integrations::clickhouse_client;
use actix::{Actor, Addr};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::timeframe::Timeframe;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio_stream::wrappers::ReceiverStream;

pub mod candlesticks_reproducer_actor;

pub struct Application {
    _candlesticks_repository: Arc<CandlesticksRepository>,
    candlesticks_reproducer_actor: Addr<CandlesticksReproducerActor>,
}

impl Application {
    pub fn new(prefetch_buffer: usize, app_config: AppConfig) -> Self {
        // infrastructure
        let clickhouse = clickhouse_client(app_config.clickhouse);
        let candlesticks_repository = Arc::new(CandlesticksRepository::new(clickhouse));

        // actors
        let candlesticks_reproducer_actor = CandlesticksReproducerActor::new(prefetch_buffer, candlesticks_repository.clone()).start();

        Application {
            _candlesticks_repository: candlesticks_repository.clone(),
            candlesticks_reproducer_actor,
        }
    }

    pub async fn start_pipeline(
        self,
        timeframes: Vec<Timeframe>,
        start_date: OffsetDateTime,
        end_date: OffsetDateTime,
    ) -> ReceiverStream<(OffsetDateTime, Vec<Candlestick>)> {
        let command = ProduceCandlesticks {
            timeframes,
            start_date,
            end_date,
            step: Duration::days(1),
        };
        let candlesticks_reproducer_actor = self.candlesticks_reproducer_actor.send(command).await.unwrap().unwrap();
        ReceiverStream::new(candlesticks_reproducer_actor)
    }
}
