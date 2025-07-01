use crate::integrations::clickhouse::candlesticks_repository::{CandlesticksReadService, CandlesticksRepository};
use crate::integrations::clickhouse::model::candlestick_row::CandlestickRow;
use crate::integrations::clickhouse::{ClickhouseRepositoryError, format_clickhouse_date};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::timeframe::Timeframe;
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;
use time::OffsetDateTime;
use tracing::error;

impl CandlesticksReadService for CandlesticksRepository {
    fn fetch_candlesticks_between(
        &self,
        timeframes: Vec<Timeframe>,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> BoxFuture<Result<Vec<Candlestick>, ClickhouseRepositoryError>> {
        let client = self.client.clone();
        let query = r#"
            SELECT
                exchange_type,
                base_asset,
                quote_asset,
                timeframe,
                open_time,
                close_time,
                open_price,
                close_price,
                low_price,
                high_price,
                volume
            FROM `candy_ass`.candlesticks
            WHERE
                timeframe IN ? AND
                open_time >= ? AND
                open_time < ?
            ORDER BY open_time ASC
        "#
        .to_string();

        async move {
            let rows = client
                .query(&query)
                .bind(timeframes)
                .bind(format_clickhouse_date(from))
                .bind(format_clickhouse_date(to))
                .fetch_all::<CandlestickRow>()
                .await
                .map_err(ClickhouseRepositoryError::from)?;

            let result = tokio_rayon::spawn(move || {
                rows.into_par_iter()
                    .map(|row| {
                        row.to_candlestick().inspect_err(|err| {
                            error!("Candlestick conversion error: {:?}", err);
                        })
                    })
                    .filter_map(|row| row.ok())
                    .collect::<Vec<_>>()
            })
            .await;
            Ok(result)
        }
        .boxed()
    }
}
