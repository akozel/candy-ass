use crate::domain::exchange_type::ExchangeType::Binance;
use crate::domain::symbol::Symbol;
use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::{BinanceSpotClient, ExchangeInfoApi};
use crate::integrations::http::utils_http::{HttpFutureExt, UrlBuilder};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfoResponse {
    pub symbols: Vec<ExchangeInfoSymbols>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfoSymbols {
    pub base_asset: String,
    pub quote_asset: String,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
}

impl ExchangeInfoResponse {
    pub fn to_symbols(&self) -> Vec<Arc<Symbol>> {
        let symbols = self.symbols.clone();
        let mut result = Vec::with_capacity(symbols.len());
        for source in symbols {
            let symbol = Symbol::from_pool(Binance, source.base_asset, source.quote_asset);
            result.push(symbol);
        }
        result
    }
}

impl ExchangeInfoApi for BinanceSpotClient {
    fn fetch_binance_exchange_info(&self) -> BoxFuture<'_, Result<(ExchangeInfoResponse, HeaderMap), HttpResponseError>> {
        let url = UrlBuilder::new(self.base_url, "/api/v3/exchangeInfo").build();

        self.client.get(url.clone()).send().parse_json_or_error().boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::exchange_type::ExchangeType::Binance;
    use crate::integrations::binance_spot_client;
    use reqwest::Client;

    #[tokio::test]
    async fn test_fetch_binance_time() {
        // Given
        let binance_client = binance_spot_client(Client::new());

        // When
        let result = binance_client.fetch_binance_exchange_info().await;

        // Then
        assert!(result.is_ok());
        let (response, _) = result.unwrap();

        let symbols = response.to_symbols();
        let target_symbol = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());

        assert!(symbols.contains(&target_symbol))
    }
}
