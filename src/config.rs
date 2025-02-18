use std::env;
use chrono::Duration;

pub struct Config {
    pub binance_ws_url: String,
    pub update_interval: Duration,
    pub volatility_window: Duration,
}

impl Config {
    pub fn new() -> Self {
        // Get update interval in seconds from env or use default (5 seconds)
        let update_seconds = env::var("UPDATE_INTERVAL_SECONDS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        // Get volatility window in hours from env or use default (6 hours)
        let window_hours = env::var("VOLATILITY_WINDOW_HOURS")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);

        Self {
            binance_ws_url: env::var("BINANCE_WS_URL")
                .expect("BINANCE_WS_URL must be set"),
            update_interval: Duration::seconds(update_seconds),
            volatility_window: Duration::hours(window_hours),
        }
    }
} 