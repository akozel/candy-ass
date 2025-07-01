use crate::domain::candlestick::Candlestick;
use crate::domain::symbol::Symbol;
use crate::domain::timeframe::Timeframe;
use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::exchange_info_api::ExchangeInfoResponse;
use crate::integrations::http::binance::spot_http_client::time_api::BinanceTimeResponse;
use axum::http::HeaderMap;
use futures_util::future::BoxFuture;
use reqwest::Client;
use std::sync::Arc;
use time::OffsetDateTime;

pub mod exchange_info_api;
pub mod klines_api;
pub mod time_api;

pub struct BinanceSpotClient {
    client: Client,
    base_url: &'static str,
}

impl BinanceSpotClient {
    pub fn new(client: Client, base_url: &'static str) -> Self {
        Self { client, base_url }
    }
}

/// Methods
pub trait ExchangeInfoApi {
    fn fetch_binance_exchange_info(&self) -> BoxFuture<'_, Result<(ExchangeInfoResponse, HeaderMap), HttpResponseError>>;
}

pub trait KlinesApi {
    fn fetch_candlesticks(
        &self,
        symbol: Arc<Symbol>,
        timeframe: Timeframe,
        limit: u16,
        start_time: Option<OffsetDateTime>,
        end_time: Option<OffsetDateTime>,
    ) -> BoxFuture<'_, Result<(Vec<Candlestick>, HeaderMap), HttpResponseError>>;
}

pub trait TimeApi {
    fn fetch_binance_time(&self) -> BoxFuture<'_, Result<(BinanceTimeResponse, HeaderMap), HttpResponseError>>;
}
