use crate::application::history_reproducer::candlesticks_reproducer_actor::errors::ReproduceHistoryError;
use crate::application::history_reproducer::candlesticks_reproducer_actor::{CandlesticksReproducerActor, Status};
use actix::{AsyncContext, Handler, Message, MessageResult, WrapFuture};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::timeframe::Timeframe;
use std::ops::Add;
use time::{Duration, OffsetDateTime};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{error, info};

#[derive(Message, Clone)]
#[rtype(result = "Result<mpsc::Receiver<(OffsetDateTime, Vec<Candlestick>)>, ReproduceHistoryError>")]
pub struct ProduceCandlesticks {
    pub timeframes: Vec<Timeframe>,
    pub start_date: OffsetDateTime,
    pub end_date: OffsetDateTime,
    pub step: Duration,
}

impl Handler<ProduceCandlesticks> for CandlesticksReproducerActor {
    type Result = MessageResult<ProduceCandlesticks>;

    fn handle(&mut self, msg: ProduceCandlesticks, ctx: &mut Self::Context) -> Self::Result {
        let timeframe = msg.timeframes.clone();
        let start_date = msg.start_date;
        let end_date = msg.end_date;

        match &self.status {
            Status::Ready => {
                let (sender, receiver) = mpsc::channel::<(OffsetDateTime, Vec<Candlestick>)>(self.prefetch_buffer);
                let candlestick_repository = self.candlesticks_read_service.clone();

                ctx.spawn(
                    async move {
                        let step = msg.step;
                        let mut start_date = start_date;
                        let mut next_date = start_date.add(step);
                        while next_date < end_date + step {
                            let start_timer = Instant::now();
                            let result = candlestick_repository
                                .fetch_candlesticks_between(timeframe.clone(), start_date, next_date)
                                .await;
                            let duration = start_timer.elapsed();

                            match result {
                                Ok(candlesticks) if candlesticks.is_empty() => break,
                                Ok(candlesticks) => {
                                    let _ = sender.send((start_date, candlesticks)).await;
                                    info!(
                                        "[CandlesticksReproducerActor] candlesticks `{}` are produced in {:?}ms (sender capacity is {})",
                                        start_date.date(),
                                        duration.as_millis(),
                                        sender.capacity()
                                    );
                                }
                                Err(err) => {
                                    error!("[CandlesticksReproducerActor] Error during candlesticks call: {}", err);
                                }
                            }
                            start_date = next_date;
                            next_date = next_date.add(msg.step);
                        }
                    }
                    .into_actor(self),
                );

                self.status = Status::Busy;
                MessageResult(Ok(receiver))
            }
            _ => MessageResult(Err(ReproduceHistoryError::ActorIsBusy)),
        }
    }
}
