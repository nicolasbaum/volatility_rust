use std::error::Error;
use tokio;
use log::{info, error};

mod price_collector;
mod volatility;
mod config;

use crate::volatility::VolatilityCalculator;
use crate::config::Config;
use crate::price_collector::{BinanceCollector, PriceAggregator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging with timestamp
    env_logger::Builder::from_default_env()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .format_target(true)
        .init();
    
    info!("Starting ETH/USDC volatility estimator...");

    // Load configuration
    dotenv::dotenv().ok();
    let config = Config::new();
    
    // Initialize collectors
    info!("Initializing Binance price collector with URL: {}", config.binance_ws_url);
    let binance = BinanceCollector::new(config.binance_ws_url.clone());
    let aggregator = PriceAggregator::new(binance);
    
    // Initialize volatility calculator with configured window
    let mut calculator = VolatilityCalculator::new(config.volatility_window);
    
    info!("Starting main loop with {} second intervals...", 
          config.update_interval.num_seconds());

    // Main program loop
    loop {
        info!("Fetching latest price...");
        match aggregator.get_aggregated_price().await {
            Ok(price) => {
                info!("Received price: ${:.2} from {} at {}", 
                    price.price, 
                    price.source,
                    price.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                calculator.add_price(price);
                if let Some(volatility) = calculator.calculate_volatility() {
                    info!("Current annualized volatility estimate: {:.2}%", volatility * 100.0);
                } else {
                    info!("Not enough data points for volatility calculation yet");
                }
            }
            Err(e) => {
                error!("Error fetching price: {}", e);
            }
        }
        
        info!("Waiting for next update...");
        tokio::time::sleep(tokio::time::Duration::from_secs(
            config.update_interval.num_seconds() as u64
        )).await;
    }
} 