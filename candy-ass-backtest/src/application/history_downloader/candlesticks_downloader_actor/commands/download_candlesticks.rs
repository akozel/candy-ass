use crate::application::history_downloader::candlesticks_downloader_actor::errors::DownloadHistoryError;
use crate::application::history_downloader::candlesticks_downloader_actor::{CandlesticksDownloaderActor, Status};
use Status::Ready;
use actix::{ActorFutureExt, AsyncContext, Handler, Message, MessageResult, WrapFuture};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::symbol::{Symbol, Symbols};
use candy_ass_core::domain::timeframe::Timeframe;
use candy_ass_core::integrations::http::binance::spot_http_client::KlinesApi;
use futures::Stream;
use futures_util::{StreamExt, stream};
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};
use tracing::{error, info, warn};

#[derive(Message, Clone)]
#[rtype(result = "Result<mpsc::Receiver<Vec<Candlestick>>, DownloadHistoryError>")]
pub struct DownloadCandlesticks {
    pub symbols: Arc<Symbols>,
    pub timeframe: Timeframe,
    pub start_date: OffsetDateTime,
    pub filter: Arc<dyn Fn(&Arc<Symbol>) -> bool + Send + Sync>,
}

impl Handler<DownloadCandlesticks> for CandlesticksDownloaderActor {
    type Result = MessageResult<DownloadCandlesticks>;

    fn handle(&mut self, msg: DownloadCandlesticks, ctx: &mut Self::Context) -> Self::Result {
        let binance_client = self.binance_client.clone();
        let concurrency = self.concurrency;
        let rate_limit = self.binance_rate_limit;
        let max_delay = rate_limit * self.concurrency;
        let buffer = self.downstream_buffer.clone();

        let timeframe = msg.timeframe.clone();
        let symbols = msg.symbols.clone();
        let start_date = msg.start_date;
        let filter = msg.filter.clone();

        match &self.status {
            Ready => {
                let (candlestick_sender, candlestick_receiver) = mpsc::channel::<Vec<Candlestick>>(buffer.clone());

                ctx.spawn(
                    async move {
                        let symbols = symbols.clone();
                        let candlestick_sender = candlestick_sender.clone();
                        let binance_client = binance_client.clone();

                        let symbol_list: Vec<Arc<Symbol>> = symbols.iter().filter(|symbol| filter(symbol)).cloned().collect();
                        let symbols_count = symbol_list.len();

                        stream::iter(symbol_list)
                            .enumerate()
                            .for_each_concurrent(concurrency, move |(index, symbol)| {
                                info!(
                                    "[CandlesticksDownloaderActor] is processing ({}/{}): {:?}",
                                    index + 1,
                                    symbols_count,
                                    symbol.short_name()
                                );
                                let candlestick_sender = candlestick_sender.clone();

                                stream_candlesticks_by_symbol(binance_client.clone(), symbol.clone(), timeframe.clone(), start_date)
                                    .then(move |(candlesticks, report)| {
                                        let candlestick_sender = candlestick_sender.clone();
                                        let capacity = candlestick_sender.capacity();

                                        if capacity < (buffer * 0.3 as usize) {
                                            warn!("[CandlesticksDownloaderActor] sender capacity is: {}; downstream is slow!", capacity);
                                        } else if capacity < (buffer * 0.5 as usize) {
                                            info!("[CandlesticksDownloaderActor] sender capacity is: {}; downstream is slow!", capacity);
                                        }

                                        async move {
                                            candlestick_sender
                                                .send(candlesticks)
                                                .await
                                                .inspect_err(|err| panic!("Candlesticks channel is closed: {:?}", err))
                                                .map(|_| report)
                                                .unwrap()
                                        }
                                    })
                                    .for_each(move |report| async move {
                                        let delay = (max_delay as u64).saturating_sub(report.latency as u64);
                                        sleep(Duration::from_millis(delay)).await;
                                    })
                            })
                            .await;
                    }
                    .into_actor(self)
                    .map(|_, act, _ctx| {
                        act.status = Ready;
                        info!("[CandlesticksDownloaderActor] is ready to work");
                    }),
                );

                self.status = Status::Busy;
                MessageResult(Ok(candlestick_receiver))
            }
            _ => MessageResult(Err(DownloadHistoryError::ActorIsBusy)),
        }
    }
}

fn stream_candlesticks_by_symbol(
    binance_client: Arc<dyn KlinesApi + Send + Sync>,
    symbol: Arc<Symbol>,
    timeframe: Timeframe,
    start_date: OffsetDateTime,
) -> impl Stream<Item = (Vec<Candlestick>, FetchReport)> {
    stream::unfold(Some(start_date), move |next_date| {
        let binance_client = binance_client.clone();
        let symbol = symbol.clone();
        let timeframe = timeframe.clone();
        async move {
            match next_date {
                Some(next_date) => {
                    let (candlesticks, report) = fetch_next_candlesticks(binance_client.clone(), symbol.clone(), timeframe, next_date).await;
                    let next_date = report.last_element_date;
                    (report.produced_count != 0).then(|| ((candlesticks, report), next_date))
                }
                None => None,
            }
        }
    })
}

pub async fn fetch_next_candlesticks(
    binance_client: Arc<dyn KlinesApi + Send + Sync>,
    symbol: Arc<Symbol>,
    timeframe: Timeframe,
    start_date: OffsetDateTime,
) -> (Vec<Candlestick>, FetchReport) {
    let timer = Instant::now();

    let (candlesticks, _headers) = binance_client
        .fetch_candlesticks(symbol.clone(), timeframe.clone(), 1000, Some(start_date), None)
        .await
        .inspect_err(|err| error!("Failed to fetch candlesticks ({:?}, {:?}, {:?}): {:?}", symbol, timeframe, start_date, err))
        .expect("Error fetching binance");

    let report = FetchReport {
        latency: timer.elapsed().as_millis() as u16,
        produced_count: candlesticks.len(),
        last_element_date: candlesticks.last().map(|last| last.close_time),
    };

    (candlesticks, report)
}

#[derive(Debug)]
pub struct FetchReport {
    pub latency: u16,
    pub produced_count: usize,
    pub last_element_date: Option<OffsetDateTime>,
}
