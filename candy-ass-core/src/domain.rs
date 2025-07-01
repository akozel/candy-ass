pub mod candlestick;
pub mod exchange_type;
pub mod symbol;
pub mod timeframe;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found")]
    NotFound,

    #[error("Unknown error")]
    Unknown,
}
