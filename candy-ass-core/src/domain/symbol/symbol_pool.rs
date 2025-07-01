use crate::domain::exchange_type::ExchangeType;
use crate::domain::symbol::Symbol;
use dashmap::DashMap;
use std::sync::{Arc, LazyLock};

pub(super) static SYMBOL_POOL: LazyLock<SymbolPool> = LazyLock::new(|| SymbolPool::new());

#[derive(Hash, Eq, PartialEq, Clone)]
pub(super) struct SymbolKey {
    pub exchange_type: ExchangeType,
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Clone)]
pub(super) struct SymbolPool {
    pool: DashMap<SymbolKey, Arc<Symbol>>,
}

impl SymbolPool {
    pub fn new() -> Self {
        SymbolPool { pool: DashMap::new() }
    }

    pub(super) fn get_or_create(&self, key: SymbolKey) -> Arc<Symbol> {
        if let Some(existing) = self.pool.get(&key) {
            return existing.value().clone();
        }

        self.pool
            .entry(key.clone())
            .or_insert_with(|| {
                Arc::new(Symbol {
                    exchange_type: key.exchange_type,
                    base_asset: key.base_asset,
                    quote_asset: key.quote_asset,
                })
            })
            .clone()
    }
}
