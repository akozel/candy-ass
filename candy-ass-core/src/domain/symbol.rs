pub mod symbol_pool;

use crate::domain::exchange_type::ExchangeType;
use crate::domain::symbol::symbol_pool::{SYMBOL_POOL, SymbolKey};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub type Symbols = Vec<Arc<Symbol>>;
pub type SymbolFilterFn = Arc<dyn Fn(&Arc<Symbol>) -> bool + Send + Sync>;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Symbol {
    pub exchange_type: ExchangeType,
    pub base_asset: String,
    pub quote_asset: String,
}

impl Symbol {
    pub fn from_pool(exchange_type: ExchangeType, base_asset: String, quote_asset: String) -> Arc<Self> {
        let key = SymbolKey {
            exchange_type,
            base_asset,
            quote_asset,
        };

        SYMBOL_POOL.get_or_create(key)
    }

    pub fn short_name(&self) -> String {
        format!("{}{}", self.base_asset, self.quote_asset).to_uppercase()
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.exchange_type == other.exchange_type && self.base_asset == other.base_asset && self.quote_asset == other.quote_asset
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.exchange_type.hash(state);
        self.base_asset.hash(state);
        self.quote_asset.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::exchange_type::ExchangeType;
    use crate::domain::symbol::Symbol;

    use ExchangeType::Binance;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn test_reuse_symbol_from_pool() {
        let s1 = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());
        let s2 = Symbol::from_pool(Binance, "BTC".to_string(), "USDT".to_string());

        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(s1.short_name(), "BTCUSDT");
    }

    #[test]
    fn test_symbol_hashing() {
        let mut set = HashSet::new();

        let s1 = Symbol {
            exchange_type: Binance,
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
        };

        let s2 = Symbol {
            exchange_type: Binance,
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
        };

        assert!(set.insert(s1.clone()));
        assert!(!set.insert(s2));
    }
}
