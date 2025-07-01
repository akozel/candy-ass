use crate::integrations::http::HttpResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FailedToFetchSymbolsError {
    #[error("Request failed: {0}")]
    Transport(#[from] HttpResponseError),
}
