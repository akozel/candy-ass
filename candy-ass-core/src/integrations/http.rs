pub mod binance;
pub mod utils_http;
pub mod utils_parser;

use reqwest::{StatusCode, Url};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpResponseError {
    #[error("Unexpected status {status} at {url}")]
    UnexpectedStatus { status: StatusCode, url: Url, body: String },

    #[error("Error at {url}: {source}")]
    UnexpectedContent {
        url: Url,
        #[source]
        source: reqwest::Error,
    },

    #[error("Unexpected error {0}")]
    Unexpected(String),

    #[error("Request failed: {0}")]
    Transport(#[from] reqwest::Error),
}
