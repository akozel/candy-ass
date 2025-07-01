use candy_ass_backtest::application::history_downloader::Application;
use candy_ass_core::domain::timeframe::Timeframe::ThreeMinutes;
use std::io;

use candy_ass_backtest::config::AppConfig;
use candy_ass_core::domain::symbol::Symbol;
use std::sync::Arc;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;
use tracing::level_filters::LevelFilter;

#[actix::main]
async fn main() {
    AppConfig::default_setup(LevelFilter::INFO);
    let config = AppConfig::from_env().expect("Failed to load application config");
    let application = Application::new(50, 14, config);

    let start_date = OffsetDateTime::parse("2023-01-01T00:00:00Z", &Rfc3339).unwrap();
    let filter: Arc<dyn Fn(&Arc<Symbol>) -> bool + Send + Sync> = Arc::new(|symbol| symbol.quote_asset == "USDT");
    application.start_pipeline(ThreeMinutes, start_date, filter).await;

    info!("Import completed. Would you like to run clickhouse optimization ? [y/n]");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let input = input.trim();

    if matches!(input, "y" | "Y") {
        info!("Starting clickhouse optimization, it may take time!");
        let _ = application.run_optimization().await;
        info!("Clickhouse optimization completed!");
    }

    println!("Press Enter, to exit...");
    let _ = BufReader::new(tokio::io::stdin()).read_line(&mut String::new()).await;
}
