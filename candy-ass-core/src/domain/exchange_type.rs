use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, EnumString, AsRefStr, EnumIter)]
pub enum ExchangeType {
    #[strum(serialize = "Binance")]
    Binance,
}

impl Display for ExchangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
