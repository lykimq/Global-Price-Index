// REST client, polling logic
use crate::exchanges::Exchange;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{PriceIndexError, Result};
use crate::models::OrderBook;
use std::time::SystemTime;

const HUOBI_BASE_URL: &str = "https://api.huobi.pro/market/depth";

#[derive(Debug, Serialize, Deserialize)]
struct HuobiOrderBook {
    bids: Vec<[f64; 2]>, // [price, volume]
    asks: Vec<[f64; 2]>, // [price, volume]
}

#[derive(Debug, Serialize, Deserialize)]
struct HuobiResponse {
    status: String,
    #[serde(rename = "err-code")]
    err_code: Option<String>,
    #[serde(rename = "err-msg")]
    err_msg: Option<String>,
    ts: i64,              // timestamp
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
            .map_err(|e| PriceIndexError::ExchangeError(format!("Failed to create HTTP client: {}", e)))?;

        // Verify the exchange is accessible by making a test request
        let params = [
            ("symbol", "btcusdt"),
            ("type", "step0"),
            ("depth", "5")  // Valid depth values: 5, 10, 20, 50, 100
        ];

        let response: HuobiResponse = client
            .get(HUOBI_BASE_URL)
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        if response.status != "ok" {
            return Err(PriceIndexError::ExchangeError(format!(
                "Huobi API error during initialization: status = {}, error = {:?}",
                response.status,
                response.err_msg
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
            ("depth", "20")  // Valid depth values: 5, 10, 20, 50, 100
        ];

        // Send the request to Huobi
        let response: HuobiResponse = self
            .client
            .get(HUOBI_BASE_URL)
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        // Check for errors
        if response.status != "ok" {
            return Err(PriceIndexError::ExchangeError(format!(
                "Huobi API error: status = {}, error = {:?}",
                response.status,
                response.err_msg
            )));
        }

        // Get the order book data
        let tick = response.tick.ok_or_else(|| {
            PriceIndexError::ExchangeError("No order book data received from Huobi".to_string())
        })?;

        // Create the order book
        Ok(OrderBook {
            bids: tick.bids
                .into_iter()
                .map(|[price, _]| [price.to_string(), "0".to_string()])
                .collect(),
            asks: tick.asks
                .into_iter()
                .map(|[price, _]| [price.to_string(), "0".to_string()])
                .collect(),
            timestamp: SystemTime::now(),
        })
    }
}
