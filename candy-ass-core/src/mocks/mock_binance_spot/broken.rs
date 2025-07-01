use crate::integrations::http::HttpResponseError::UnexpectedStatus;
use crate::mocks::mock_binance_spot::MockBinanceSpotClient;
use axum::http::StatusCode;
use reqwest::Url;
use std::sync::{Arc, LazyLock};

pub static BROKEN_BINANCE_SPOT_CLIENT: LazyLock<Arc<MockBinanceSpotClient>> = LazyLock::new(|| {
    let mut binance_spot_client = MockBinanceSpotClient::new();
    binance_spot_client.expect_fetch_binance_exchange_info().returning(move || {
        Box::pin(async move {
            Err(UnexpectedStatus {
                status: StatusCode::NOT_FOUND,
                url: Url::parse("http://fake").unwrap(),
                body: "fake error".into(),
            })
        })
    });
    binance_spot_client.expect_fetch_candlesticks().returning(move |_, _, _, _, _| {
        Box::pin(async move {
            Err(UnexpectedStatus {
                status: StatusCode::NOT_FOUND,
                url: Url::parse("http://fake").unwrap(),
                body: "fake error".into(),
            })
        })
    });
    Arc::new(binance_spot_client)
});
