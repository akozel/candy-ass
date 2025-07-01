use candy_ass_backtest::application::history_reproducer::Application;
use futures_util::StreamExt;

use candy_ass_backtest::config::AppConfig;

use candy_ass_core::domain::timeframe::Timeframe::ThreeMinutes;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::Instant;
use tracing::info;
use tracing::level_filters::LevelFilter;

#[actix::main]
async fn main() {
    AppConfig::default_setup(LevelFilter::INFO);
    let config = AppConfig::from_env().expect("Failed to load application config");
    let application = Application::new(4, config);

    let timeframes = vec![ThreeMinutes];
    let start_date = OffsetDateTime::parse("2024-01-01T00:00:00Z", &Rfc3339).unwrap();
    let end_date = OffsetDateTime::parse("2024-02-01T00:00:00Z", &Rfc3339).unwrap();

    let timer = Instant::now();
    let pipeline = application.start_pipeline(timeframes, start_date, end_date).await;
    let processed = pipeline
        .flat_map(|(_date_time, candlesticks)| futures::stream::iter(candlesticks))
        .count()
        .await;

    let duration = timer.elapsed();

    info!("Backtest completed in {:?}, candlesticks processed: {:?}", duration, processed);
    println!("Press Enter, to exit...");
    let _ = BufReader::new(tokio::io::stdin()).read_line(&mut String::new()).await;
}
