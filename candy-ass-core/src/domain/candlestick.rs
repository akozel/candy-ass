use crate::domain::symbol::Symbol;
use crate::domain::timeframe::Timeframe;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Candlestick {
    pub symbol: Arc<Symbol>,
    pub timeframe: Timeframe,
    #[serde(with = "time::serde::rfc3339")]
    pub open_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub close_time: OffsetDateTime,
    pub open_price: f64,
    pub close_price: f64,
    pub low_price: f64,
    pub high_price: f64,
    pub volume: f64,
}
