use crate::domain::candlestick::Candlestick;
use crate::domain::exchange_type::ExchangeType;
use crate::domain::exchange_type::ExchangeType::Binance;
use crate::domain::symbol::Symbol;
use crate::domain::timeframe::Timeframe::OneDay;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};
use tokio::fs;

pub static BTC_USDT_CANDLESTICK: LazyLock<Candlestick> = LazyLock::new(|| {
    let btc_usdt = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
    let start_date = OffsetDateTime::parse("2024-01-01T00:00:00Z", &Rfc3339).unwrap();
    Candlestick {
        symbol: btc_usdt,
        timeframe: OneDay,
        open_time: start_date,
        close_time: start_date + Duration::days(1),
        open_price: 100_000.0,
        close_price: 101_000.0,
        high_price: 101_000.001,
        low_price: 99_999.999,
        volume: 1_111.1,
    }
});

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")        // <-- нет «\» – только join
        .join("mocks")
        .join("fixtures")
        .join(filename)
}

async fn fixture_string(filename: &str) -> String {
    tokio::fs::read_to_string(fixture_path(filename))
        .await
        .expect("can't read fixture")
}

pub async fn mock_candlesticks(symbol: Arc<Symbol>) -> Option<Vec<Candlestick>> {
    let path = format!("candlesticks/{}_{}_{}.json", symbol.exchange_type, symbol.base_asset, symbol.quote_asset).to_lowercase();
    serde_json::from_str(fixture_string(path.as_ref()).await.as_ref()).ok()
}

pub async fn mock_symbols(exchange_type: ExchangeType) -> Vec<Arc<Symbol>> {
    vec![
        Symbol::from_pool(exchange_type.clone(), "BTC".into(), "USDT".into()),
        Symbol::from_pool(exchange_type.clone(), "ETH".into(), "USDT".into()),
    ]
}
