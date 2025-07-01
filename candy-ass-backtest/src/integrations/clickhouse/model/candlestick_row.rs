use crate::integrations::clickhouse::ClickhouseRepositoryError;
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::symbol::Symbol;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct CandlestickRow {
    pub exchange_type: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub timeframe: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub open_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub close_time: OffsetDateTime,
    pub open_price: f64,
    pub close_price: f64,
    pub low_price: f64,
    pub high_price: f64,
    pub volume: f64,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct GroupedCandlestickRow {
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub open_time: OffsetDateTime,
    pub candlestick_rows: Vec<CandlestickRow>,
}

impl CandlestickRow {
    pub fn to_candlestick(self) -> Result<Candlestick, ClickhouseRepositoryError> {
        let exchange_type = self.exchange_type.as_str().try_into()?;
        let symbol = Symbol::from_pool(exchange_type, self.base_asset, self.quote_asset);
        let timeframe = self.timeframe.as_str().try_into()?;
        Ok(Candlestick {
            symbol,
            timeframe,
            open_time: self.open_time,
            close_time: self.close_time,
            open_price: self.open_price,
            close_price: self.close_price,
            low_price: self.low_price,
            high_price: self.high_price,
            volume: self.volume,
        })
    }
}

impl From<&Candlestick> for CandlestickRow {
    fn from(src: &Candlestick) -> Self {
        Self {
            exchange_type: src.symbol.exchange_type.to_string(),
            base_asset: src.symbol.base_asset.clone(),
            quote_asset: src.symbol.quote_asset.clone(),
            timeframe: src.timeframe.to_string(),
            open_time: src.open_time,
            close_time: src.close_time,
            open_price: src.open_price,
            close_price: src.close_price,
            low_price: src.low_price,
            high_price: src.high_price,
            volume: src.volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExchangeType::Binance;
    use candy_ass_core::domain::exchange_type::ExchangeType;
    use candy_ass_core::domain::symbol::Symbol;
    use candy_ass_core::domain::timeframe::Timeframe;
    use std::str::FromStr;
    use time::OffsetDateTime;

    fn make_candlestick_row() -> CandlestickRow {
        CandlestickRow {
            exchange_type: "Binance".to_string(),
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
            timeframe: "1m".to_string(),
            open_time: OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap(),
            close_time: OffsetDateTime::from_unix_timestamp(1_600_000_060).unwrap(),
            open_price: 10000.0,
            close_price: 10100.0,
            low_price: 9950.0,
            high_price: 10200.0,
            volume: 0.25,
        }
    }

    fn make_candlestick() -> Candlestick {
        let symbol = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        Candlestick {
            symbol,
            timeframe: Timeframe::from_str("1m").unwrap(),
            open_time: OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap(),
            close_time: OffsetDateTime::from_unix_timestamp(1_600_000_060).unwrap(),
            open_price: 10000.0,
            close_price: 10100.0,
            low_price: 9950.0,
            high_price: 10200.0,
            volume: 0.25,
        }
    }

    #[test]
    fn test_to_candlestick() {
        let row = make_candlestick_row();
        let candlestick = row.to_candlestick().unwrap();
        assert_eq!(candlestick.open_price, 10000.0);
        assert_eq!(candlestick.close_price, 10100.0);
        assert_eq!(candlestick.volume, 0.25);
        assert_eq!(candlestick.symbol.base_asset, "BTC");
        assert_eq!(candlestick.symbol.exchange_type, Binance);
        assert_eq!(candlestick.timeframe, Timeframe::from_str("1m").unwrap());
    }

    #[test]
    fn test_from_candlestick() {
        let candlestick = make_candlestick();
        let row = CandlestickRow::from(&candlestick);

        assert_eq!(row.base_asset, "BTC");
        assert_eq!(row.quote_asset, "USDT");
        assert_eq!(row.exchange_type, "Binance");
        assert_eq!(row.timeframe, "1m");
        assert_eq!(row.open_price, 10000.0);
        assert_eq!(row.close_price, 10100.0);
    }

    #[test]
    fn test_serialization_and_deserialization() {
        let row = make_candlestick_row();
        let json = serde_json::to_string(&row).unwrap();
        let row2: CandlestickRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row.base_asset, row2.base_asset);
        assert_eq!(row.open_time, row2.open_time);
        assert_eq!(row.volume, row2.volume);
    }
}
