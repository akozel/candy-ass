pub mod broken;
pub mod default;

use crate::domain::candlestick::Candlestick;
use crate::domain::symbol::Symbol;
use crate::domain::timeframe::Timeframe;
use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::exchange_info_api::ExchangeInfoResponse;
use crate::integrations::http::binance::spot_http_client::time_api::BinanceTimeResponse;
use crate::integrations::http::binance::spot_http_client::{ExchangeInfoApi, KlinesApi, TimeApi};
use crate::integrations::http::binance::{BINANCE_HEADER_USED_WEIGHT, BINANCE_HEADER_USED_WEIGHT_1M};
use axum::http::HeaderMap;
use futures_util::future::BoxFuture;
use mockall::mock;
use reqwest::header::HeaderValue;
use std::sync::{Arc, LazyLock};
use time::OffsetDateTime;

pub static HEADER_MAP: LazyLock<HeaderMap> = LazyLock::new(|| {
    let mut value = HeaderMap::new();
    value.append(BINANCE_HEADER_USED_WEIGHT, HeaderValue::from_static("9999"));
    value.append(BINANCE_HEADER_USED_WEIGHT_1M, HeaderValue::from_static("9999"));
    value
});

mock! {
    pub BinanceSpotClient {}

    impl ExchangeInfoApi for BinanceSpotClient {
        fn fetch_binance_exchange_info(
            &self,
        ) -> BoxFuture<'_, Result<(ExchangeInfoResponse, HeaderMap), HttpResponseError>>;
    }
    impl KlinesApi for BinanceSpotClient {
        fn fetch_candlesticks(
            &self,
            symbol: Arc<Symbol>,
            timeframe: Timeframe,
            limit: u16,
            start_time: Option<OffsetDateTime>,
            end_time: Option<OffsetDateTime>,
        ) -> BoxFuture<'_, Result<(Vec<Candlestick>, HeaderMap), HttpResponseError>>;
    }
    impl TimeApi for BinanceSpotClient {
        fn fetch_binance_time(&self) -> BoxFuture<'_, Result<(BinanceTimeResponse, HeaderMap), HttpResponseError>>;
    }
}
