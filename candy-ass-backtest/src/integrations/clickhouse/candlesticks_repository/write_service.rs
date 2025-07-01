use crate::integrations::clickhouse::ClickhouseRepositoryError;
use crate::integrations::clickhouse::candlesticks_repository::{CandlesticksRepository, CandlesticksWriteService};
use crate::integrations::clickhouse::model::candlestick_row::CandlestickRow;
use candy_ass_core::domain::candlestick::Candlestick;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, StreamExt, TryFutureExt};

impl CandlesticksWriteService for CandlesticksRepository {
    fn init(&self) -> BoxFuture<Result<(), ClickhouseRepositoryError>> {
        futures::future::ready(())
            .then(|_| {
                let create_database_query = "CREATE DATABASE IF NOT EXISTS `candy_ass`";
                self.client.query(create_database_query).execute()
            })
            .and_then(|_| {
                let create_table_query = r#"
                        CREATE TABLE IF NOT EXISTS `candy_ass`.candlesticks
                        (
                            exchange_type LowCardinality(String),
                            base_asset LowCardinality(String),
                            quote_asset LowCardinality(String),
                            timeframe LowCardinality(String),
                            open_time DateTime,
                            close_time DateTime,
                            open_price Float64,
                            close_price Float64,
                            low_price Float64,
                            high_price Float64,
                            volume Float64
                        )
                        ENGINE = ReplacingMergeTree(volume)
                        PRIMARY KEY (open_time, timeframe, exchange_type, base_asset, quote_asset)
                        ORDER BY (open_time, timeframe, exchange_type, base_asset, quote_asset)
                        SETTINGS index_granularity = 8192;
                "#;
                self.client.query(create_table_query).execute()
            })
            .map_err(ClickhouseRepositoryError::from)
            .boxed()
    }

    fn bulk_insert_candlesticks(&self, chunk: Vec<Vec<Candlestick>>) -> BoxFuture<Result<(), ClickhouseRepositoryError>> {
        async move {
            let mut insert = self
                .client
                .insert("`candy_ass`.candlesticks")
                .expect("[CandlesticksRepository] failed to open insert statement for clickhouse");

            let rows = tokio_stream::iter(chunk)
                .flat_map_unordered(8, |candlesticks| tokio_stream::iter(candlesticks).map(|x| CandlestickRow::from(&x)))
                .collect::<Vec<_>>()
                .await;

            for row in rows {
                insert.write(&row).await.expect("[CandlesticksRepository] failed to append candlestick row");
            }

            insert
                .end()
                .await
                .expect("[CandlesticksRepository] failed to complete bulk insert for candlesticks table");
            Ok(())
        }
        .boxed()
    }

    fn run_optimization(&self) -> BoxFuture<Result<(), ClickhouseRepositoryError>> {
        async move {
            let optimization_query = "OPTIMIZE TABLE `candy_ass`.candlesticks FINAL";
            self.client.query(optimization_query).execute().await.map_err(ClickhouseRepositoryError::from)
        }
        .boxed()
    }
}
