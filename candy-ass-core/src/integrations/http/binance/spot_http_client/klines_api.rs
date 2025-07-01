use crate::domain::candlestick::Candlestick;
use crate::domain::symbol::Symbol;
use crate::domain::timeframe::Timeframe;
use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::{BinanceSpotClient, KlinesApi};
use crate::integrations::http::utils_http::{HttpFutureExt, UrlBuilder};
use crate::integrations::http::utils_parser::{parse_f64, parse_u64};
use crate::utils::OffsetDateTimeExt;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt, future};
use reqwest::header::HeaderMap;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct KlineResponse {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
    pub quote_asset_volume: f64,
    pub number_of_trades: u64,
    pub taker_buy_base_volume: f64,
    pub taker_buy_quote_volume: f64,
}

impl KlineResponse {
    pub fn into_candlestick(self, symbol: Arc<Symbol>, timeframe: Timeframe) -> Candlestick {
        let open_time = OffsetDateTime::from_unix_timestamp_millis(self.open_time as i64).ok().unwrap();
        let close_time = OffsetDateTime::from_unix_timestamp_millis(self.close_time as i64).ok().unwrap();

        Candlestick {
            symbol,
            timeframe,
            open_time,
            close_time,
            open_price: self.open,
            close_price: self.close,
            low_price: self.low,
            high_price: self.high,
            volume: self.volume,
        }
    }
}

impl KlinesApi for BinanceSpotClient {
    fn fetch_candlesticks(
        &self,
        symbol: Arc<Symbol>,
        timeframe: Timeframe,
        limit: u16,
        start_time: Option<OffsetDateTime>,
        end_time: Option<OffsetDateTime>,
    ) -> BoxFuture<Result<(Vec<Candlestick>, HeaderMap), HttpResponseError>> {
        let url = UrlBuilder::new(self.base_url, "/api/v3/klines")
            .with_param("symbol", symbol.short_name())
            .with_param("interval", timeframe.as_ref())
            .with_param("limit", limit)
            .with_optional_param("startTime", start_time.map(|t| t.unix_timestamp_millis()).as_ref())
            .with_optional_param("endTime", end_time.map(|t| t.unix_timestamp_millis()).as_ref())
            .build();

        self.client
            .get(url.clone())
            .send()
            .parse_json_or_error::<Vec<Vec<serde_json::Value>>>()
            .and_then(move |(raw, headers)| {
                match raw
                    .into_iter()
                    .map(|json_arr| Candlestick::try_from_json_array(json_arr, symbol.clone(), timeframe.clone()))
                    .collect::<Result<Vec<Candlestick>, _>>()
                {
                    Ok(candlesticks) => future::ready(Ok((candlesticks, headers))),
                    Err(err) => future::ready(Err(err)),
                }
            })
            .boxed()
    }
}

pub trait CandlestickTryFromJsonArray {
    fn try_from_json_array(raw: Vec<serde_json::Value>, symbol: Arc<Symbol>, timeframe: Timeframe) -> Result<Self, HttpResponseError>
    where
        Self: Sized;
}

impl CandlestickTryFromJsonArray for Candlestick {
    fn try_from_json_array(raw: Vec<serde_json::Value>, symbol: Arc<Symbol>, timeframe: Timeframe) -> Result<Self, HttpResponseError> {
        if raw.len() < 11 {
            return Err(HttpResponseError::Unexpected("Not enough fields".into()));
        }

        let open_time = OffsetDateTime::from_unix_timestamp_millis(parse_u64(&raw[0], "open_time")? as i64).unwrap();

        let close_time = OffsetDateTime::from_unix_timestamp_millis(parse_u64(&raw[6], "close_time")? as i64).unwrap();

        Ok(Candlestick {
            symbol,
            timeframe,
            open_time,
            close_time,
            open_price: parse_f64(&raw[1], "open")?,
            high_price: parse_f64(&raw[2], "high")?,
            low_price: parse_f64(&raw[3], "low")?,
            close_price: parse_f64(&raw[4], "close")?,
            volume: parse_f64(&raw[5], "volume")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::exchange_type::ExchangeType::Binance;
    use crate::domain::timeframe::Timeframe::ThreeMinutes;
    use crate::integrations::binance_spot_client;
    use crate::integrations::http::binance::{BINANCE_HEADER_USED_WEIGHT, BINANCE_HEADER_USED_WEIGHT_1M};
    use reqwest::Client;

    #[tokio::test]
    async fn fetch_binance_candlesticks_test() {
        // Given
        let binance_client = binance_spot_client(Client::new());
        let symbol = Symbol::from_pool(Binance, "ETH".to_string(), "USDT".to_string());
        let timeframe = ThreeMinutes;

        // When
        let result = binance_client.fetch_candlesticks(symbol, timeframe, 3, None, None).await;

        // Then
        let (result, headers) = result.unwrap();

        assert_eq!(result.len(), 3);
        assert!(result[0].open_time < result[2].open_time);

        assert!(headers.contains_key(BINANCE_HEADER_USED_WEIGHT));
        assert!(headers.contains_key(BINANCE_HEADER_USED_WEIGHT_1M));
    }

    #[tokio::test]
    async fn fetch_binance_kline_error_test() {
        // Given
        let binance_client = binance_spot_client(Client::new());
        let symbol = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        let timeframe = ThreeMinutes;

        // When
        let result = binance_client.fetch_candlesticks(symbol, timeframe, 0, None, None).await;

        // Then
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn invalid_kline_data_test() {
        let invalid_data = vec![serde_json::json!(null); 10];
        let symbol = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        let timeframe = ThreeMinutes;

        let result = Candlestick::try_from_json_array(invalid_data, symbol, timeframe);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn into_candlestick_success_test() {
        let kline = KlineResponse {
            open_time: 1_682_544_000_000,  // 2023-05-01T00:00:00Z in millis
            close_time: 1_682_547_800_000, // 2023-05-01T01:03:00Z in millis
            open: 28000.0,
            high: 28500.0,
            low: 27900.0,
            close: 28300.0,
            volume: 123.45,
            quote_asset_volume: 0.0,
            number_of_trades: 0,
            taker_buy_base_volume: 0.0,
            taker_buy_quote_volume: 0.0,
        };

        let symbol = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        let timeframe = ThreeMinutes;

        let candlestick = kline.into_candlestick(symbol, timeframe);

        assert_eq!(candlestick.symbol.exchange_type, Binance);
        assert_eq!(candlestick.symbol.base_asset, "BTC".to_string());
        assert_eq!(candlestick.symbol.quote_asset, "USDT".to_string());

        assert_eq!(candlestick.timeframe, ThreeMinutes);
        assert_eq!(candlestick.open_price, 28000.0);
        assert_eq!(candlestick.high_price, 28500.0);
        assert_eq!(candlestick.low_price, 27900.0);
        assert_eq!(candlestick.close_price, 28300.0);
        assert_eq!(candlestick.volume, 123.45);

        assert_eq!(candlestick.open_time, OffsetDateTime::from_unix_timestamp_millis(1_682_544_000_000).unwrap());
        assert_eq!(candlestick.close_time, OffsetDateTime::from_unix_timestamp_millis(1_682_547_800_000).unwrap());
    }
}
