// REST client, polling logic
use crate::error::{PriceIndexError, Result};
use crate::exchanges::Exchange;
use crate::models::{Order, OrderBook};
use async_trait::async_trait;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::SystemTime;

// Load environment variable with fallback
fn get_huobi_url() -> String {
    dotenv().ok();
    env::var("HUOBI_URL").unwrap_or_else(|_| "https://api.huobi.pro/market/depth".to_string())
}

// API Huobi returns [price: float, quantity: float]
#[derive(Debug, Serialize, Deserialize)]
struct HuobiOrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HuobiResponse {
    status: String,
    #[serde(rename = "err-code")]
    err_code: Option<String>,
    #[serde(rename = "err-msg")]
    err_msg: Option<String>,
    ts: i64,                      // timestamp
    tick: Option<HuobiOrderBook>, // order book
}

pub struct HuobiExchange {
    client: reqwest::Client,
}

impl HuobiExchange {
    pub async fn new() -> Result<Self> {
        // Create a new client with custom configuration
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| {
                PriceIndexError::ExchangeError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Verify the exchange is accessible by making a test request
        let params = [
            ("symbol", "btcusdt"),
            ("type", "step0"),
            ("depth", "5"), // Valid depth values: 5, 10, 20, 50, 100
        ];

        let response: HuobiResponse = client
            .get(&get_huobi_url())
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        if response.status != "ok" {
            return Err(PriceIndexError::ExchangeError(format!(
                "Huobi API error during initialization: status = {}, error = {:?}",
                response.status, response.err_msg
            )));
        }

        Ok(Self { client })
    }
}

#[async_trait]
impl Exchange for HuobiExchange {
    fn name(&self) -> &'static str {
        "Huobi"
    }

    // Fetch the current order book
    async fn fetch_order_book(&self) -> Result<OrderBook> {
        // Define the parameters for the request
        let params = [
            ("symbol", "btcusdt"),
            ("type", "step0"),
            ("depth", "20"), // Valid depth values: 5, 10, 20, 50, 100
        ];

        // Send the request to Huobi
        let response: HuobiResponse = self
            .client
            .get(&get_huobi_url())
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        // Check for errors
        if response.status != "ok" {
            return Err(PriceIndexError::ExchangeError(format!(
                "Huobi API error: status = {}, error = {:?}",
                response.status, response.err_msg
            )));
        }

        // Get the order book data
        let tick = response.tick.ok_or_else(|| {
            PriceIndexError::ExchangeError("No order book data received from Huobi".to_string())
        })?;

        // Create the order book
        Ok(OrderBook {
            bids: tick
                .bids
                .into_iter()
                .map(|order| Order {
                    price: order.price,
                    quantity: order.quantity,
                })
                .collect(),
            asks: tick
                .asks
                .into_iter()
                .map(|order| Order {
                    price: order.price,
                    quantity: order.quantity,
                })
                .collect(),
            timestamp: SystemTime::now(),
        })
    }
}
