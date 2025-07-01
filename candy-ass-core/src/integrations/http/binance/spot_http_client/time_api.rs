use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::{BinanceSpotClient, TimeApi};
use crate::integrations::http::utils_http::{HttpFutureExt, UrlBuilder};
use futures::FutureExt;
use futures::future::BoxFuture;
use reqwest::header::HeaderMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceTimeResponse {
    pub server_time: u128,
}

impl TimeApi for BinanceSpotClient {
    fn fetch_binance_time(&self) -> BoxFuture<'_, Result<(BinanceTimeResponse, HeaderMap), HttpResponseError>> {
        let url = UrlBuilder::new(self.base_url, "/api/v3/time").build();

        self.client.get(url.clone()).send().parse_json_or_error().boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::binance_spot_client;
    use crate::integrations::http::binance::{BINANCE_HEADER_USED_WEIGHT, BINANCE_HEADER_USED_WEIGHT_1M};
    use reqwest::Client;
    use std::ops::Sub;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_fetch_binance_time() {
        // Given
        let binance_client = binance_spot_client(Client::new());

        // When
        let result = binance_client.fetch_binance_time().await;

        // Then
        assert!(result.is_ok());
        let (response, headers) = result.unwrap();

        let assert_time = SystemTime::now()
            .sub(Duration::from_secs(60 * 3))
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        assert!(assert_time < response.server_time);

        assert!(headers.contains_key(BINANCE_HEADER_USED_WEIGHT));
        assert!(headers.contains_key(BINANCE_HEADER_USED_WEIGHT_1M));
    }
}
