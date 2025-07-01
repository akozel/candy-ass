use crate::domain::candlestick::Candlestick;
use crate::domain::symbol::Symbol;
use crate::integrations::http::HttpResponseError;
use crate::integrations::http::binance::spot_http_client::exchange_info_api::{ExchangeInfoResponse, ExchangeInfoSymbols};
use crate::mocks::fixtures::mock_candlesticks;
use crate::mocks::mock_binance_spot::{HEADER_MAP, MockBinanceSpotClient};
use axum::http::HeaderMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::time::sleep;

pub static DEFAULT_BINANCE_SPOT_CLIENT: LazyLock<Arc<MockBinanceSpotClient>> = LazyLock::new(|| {
    let mut binance_spot_client = MockBinanceSpotClient::new();
    binance_spot_client.expect_fetch_binance_exchange_info().returning(move || {
        Box::pin(async move {
            sleep(Duration::from_millis(3)).await;
            fake_exchange_info_response()
        })
    });
    binance_spot_client
        .expect_fetch_candlesticks()
        .returning(move |symbol, _, _, _, _| Box::pin(async move { fake_candlesticks(symbol).await }));
    Arc::new(binance_spot_client)
});

pub fn fake_exchange_info_response() -> Result<(ExchangeInfoResponse, HeaderMap), HttpResponseError> {
    let btc_usdt = ExchangeInfoSymbols {
        base_asset: "BTC".into(),
        quote_asset: "USDT".into(),
        is_spot_trading_allowed: true,
        is_margin_trading_allowed: true,
    };
    let eth_usdt = ExchangeInfoSymbols {
        base_asset: "ETH".into(),
        quote_asset: "USDT".into(),
        is_spot_trading_allowed: true,
        is_margin_trading_allowed: true,
    };
    let result = ExchangeInfoResponse {
        symbols: vec![btc_usdt, eth_usdt],
    };
    Ok((result, HEADER_MAP.clone()))
}

async fn fake_candlesticks(symbol: Arc<Symbol>) -> Result<(Vec<Candlestick>, HeaderMap), HttpResponseError> {
    let result = mock_candlesticks(symbol).await;

    match result {
        Some(result) => Ok((result, HEADER_MAP.clone())),
        None => Ok((vec![], HEADER_MAP.clone())),
    }
}
