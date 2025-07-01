pub mod spot_http_client;

pub const BINANCE_SPOT_BASE_URL: &str = "https://api.binance.com/api";
pub const BINANCE_RATE_LIMIT: usize = 25;

pub const BINANCE_HEADER_USED_WEIGHT: &str = "x-mbx-used-weight";
pub const BINANCE_HEADER_USED_WEIGHT_1M: &str = "x-mbx-used-weight-1m";
