use std::error::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tungstenite::{connect, Message};
use url::Url;
use async_trait::async_trait;
use log;
use tokio::sync::Mutex;

#[cfg(feature = "uniswap")]
use web3::{
    types::{H160, U256},
    contract::{Contract, Options},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub source: String,
}

#[async_trait]
pub trait PriceCollector {
    async fn get_latest_price(&self) -> Result<PricePoint, Box<dyn Error>>;
}

// Uniswap collector behind feature flag
#[cfg(feature = "uniswap")]
pub struct UniswapCollector {
    pool_address: H160,
    web3_client: web3::Web3<web3::transports::Http>,
}

#[cfg(feature = "uniswap")]
const UNISWAP_V3_POOL_ABI: &[u8] = include_bytes!("../abi/uniswap_v3_pool.json");

#[cfg(feature = "uniswap")]
impl UniswapCollector {
    pub fn new(pool_address: H160, web3_client: web3::Web3<web3::transports::Http>) -> Self {
        Self {
            pool_address,
            web3_client,
        }
    }

    async fn get_slot0(&self) -> Result<(U256, i32, u16, u16, u16, u8, bool), Box<dyn Error>> {
        let contract = Contract::from_json(
            self.web3_client.eth(),
            self.pool_address,
            UNISWAP_V3_POOL_ABI,
        )?;

        let result: (U256, i32, u16, u16, u16, u8, bool) = contract
            .query("slot0", (), None, Options::default(), None)
            .await?;

        Ok(result)
    }
}

#[cfg(feature = "uniswap")]
#[async_trait]
impl PriceCollector for UniswapCollector {
    async fn get_latest_price(&self) -> Result<PricePoint, Box<dyn Error>> {
        let (sqrt_price_x96, _, _, _, _, _, _) = self.get_slot0().await?;
        
        // Convert sqrtPriceX96 to actual price
        let price = (sqrt_price_x96.as_u128() as f64).powi(2) / 2.0_f64.powi(192);
        
        Ok(PricePoint {
            timestamp: Utc::now(),
            price,
            source: "Uniswap".to_string(),
        })
    }
}

pub struct BinanceCollector {
    websocket_url: String,
    socket: Mutex<Option<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>>,
}

#[derive(Debug, Deserialize)]
struct BinanceTradeEvent {
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "T")]
    timestamp: i64,
}

impl BinanceCollector {
    pub fn new(websocket_url: String) -> Self {
        Self { 
            websocket_url,
            socket: Mutex::new(None),
        }
    }

    async fn ensure_connection(&self) -> Result<(), Box<dyn Error>> {
        let mut socket_guard = self.socket.lock().await;
        if socket_guard.is_none() {
            log::info!("Establishing new Binance WebSocket connection...");
            let (mut ws_stream, _) = connect(Url::parse(&self.websocket_url)?)?;
            
            // Subscribe to trade stream
            let subscribe_msg = r#"{"method": "SUBSCRIBE", "params": ["ethusdc@trade"], "id": 1}"#;
            log::debug!("Sending subscription message: {}", subscribe_msg);
            ws_stream.write_message(Message::Text(subscribe_msg.into()))?;

            // Read subscription confirmation
            let conf_msg = ws_stream.read_message()?;
            log::debug!("Received subscription confirmation: {:?}", conf_msg);

            *socket_guard = Some(ws_stream);
        }
        Ok(())
    }
}

#[async_trait]
impl PriceCollector for BinanceCollector {
    async fn get_latest_price(&self) -> Result<PricePoint, Box<dyn Error>> {
        self.ensure_connection().await?;
        
        let mut socket_guard = self.socket.lock().await;
        if let Some(socket) = socket_guard.as_mut() {
            loop {
                match socket.read_message() {
                    Ok(Message::Text(msg)) => {
                        log::debug!("Received message: {}", msg);
                        
                        // Try to parse the message
                        if let Ok(trade) = serde_json::from_str::<BinanceTradeEvent>(&msg) {
                            let price_point = PricePoint {
                                timestamp: DateTime::from_timestamp(trade.timestamp / 1000, 0)
                                    .unwrap_or_else(|| Utc::now()),
                                price: trade.price.parse()?,
                                source: "Binance".to_string(),
                            };
                            log::debug!("Parsed price point: {:?}", price_point);
                            return Ok(price_point);
                        }
                    }
                    Ok(msg) => {
                        log::debug!("Received non-text message: {:?}", msg);
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        // Clear the socket so we'll reconnect next time
                        *socket_guard = None;
                        return Err(e.into());
                    }
                }
            }
        } else {
            return Err("WebSocket connection not established".into());
        }
    }
}

pub struct PriceAggregator {
    binance: BinanceCollector,
    #[cfg(feature = "uniswap")]
    uniswap: Option<UniswapCollector>,
}

impl PriceAggregator {
    #[cfg(not(feature = "uniswap"))]
    pub fn new(binance: BinanceCollector) -> Self {
        Self { binance }
    }

    #[cfg(feature = "uniswap")]
    pub fn new(binance: BinanceCollector, uniswap: Option<UniswapCollector>) -> Self {
        Self { binance, uniswap }
    }

    pub async fn get_aggregated_price(&self) -> Result<PricePoint, Box<dyn Error>> {
        #[cfg(feature = "uniswap")]
        if let Some(uniswap) = &self.uniswap {
            match uniswap.get_latest_price().await {
                Ok(uni_price) => {
                    match self.binance.get_latest_price().await {
                        Ok(bin_price) => {
                            return Ok(PricePoint {
                                timestamp: Utc::now(),
                                price: (uni_price.price + bin_price.price) / 2.0,
                                source: "Aggregated".to_string(),
                            });
                        }
                        Err(e) => {
                            log::error!("Binance price collection failed: {}", e);
                            return Ok(uni_price);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Uniswap price collection failed: {}", e);
                    return self.binance.get_latest_price().await;
                }
            }
        }

        // If Uniswap is not enabled or not configured, use only Binance
        self.binance.get_latest_price().await
    }
} 