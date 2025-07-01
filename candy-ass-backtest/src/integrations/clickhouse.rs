use clickhouse::error::Error;
use strum::ParseError;
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub mod candlesticks_repository;
pub mod model;

/// Utils
fn format_clickhouse_date(dt: OffsetDateTime) -> String {
    let rfc3339 = dt.format(&Rfc3339).unwrap();
    let mut s = rfc3339[..rfc3339.len() - 1].to_string();
    s.replace_range(10..11, " ");
    s.truncate(19);
    s
}

/// Errors
#[derive(Debug, Error)]
pub enum ClickhouseRepositoryError {
    #[error("Failed to process clickhouse query: {0}")]
    UnexpectedResult(#[from] Error),

    #[error("Failed to parse a row: {0}")]
    ParsingError(#[from] ParseError),
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    #[test]
    fn test_format_clickhouse() {
        // given
        let dt = OffsetDateTime::parse("2024-01-01T02:03:04Z", &Rfc3339).unwrap();

        // when
        let formatted = format_clickhouse_date(dt);

        // then
        assert_eq!(formatted, "2024-01-01 02:03:04");
    }
}
