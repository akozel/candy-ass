use crate::integrations::http::binance::BINANCE_SPOT_BASE_URL;
use crate::integrations::http::binance::spot_http_client::BinanceSpotClient;
use std::sync::Arc;

pub mod http;

pub fn binance_spot_client(client: reqwest::Client) -> Arc<BinanceSpotClient> {
    let client = BinanceSpotClient::new(client, BINANCE_SPOT_BASE_URL);
    Arc::new(client)
}
