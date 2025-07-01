use crate::integrations::clickhouse::ClickhouseRepositoryError;
use crate::integrations::clickhouse::candlesticks_repository::{CandlesticksReadService, CandlesticksWriteService};
use candy_ass_core::domain::candlestick::Candlestick;
use candy_ass_core::domain::timeframe::Timeframe;
use futures_util::future::BoxFuture;
use mockall::mock;
use time::OffsetDateTime;

mock! {
    pub Clickhouse {}

    impl CandlesticksWriteService for Clickhouse {
        fn init(&self) -> BoxFuture<'static, Result<(), ClickhouseRepositoryError>>;
        fn bulk_insert_candlesticks(
            &self,
            chunk: Vec<Vec<Candlestick>>,
        ) -> BoxFuture<'static, Result<(), ClickhouseRepositoryError>>;
        fn run_optimization(&self) -> BoxFuture<'static, Result<(), ClickhouseRepositoryError>>;
    }
    impl CandlesticksReadService for Clickhouse {
        fn fetch_candlesticks_between(
            &self,
            timeframe: Vec<Timeframe>,
            from: OffsetDateTime,
            to: OffsetDateTime,) -> BoxFuture<'static, Result<Vec<Candlestick>, ClickhouseRepositoryError>>;
    }
}
