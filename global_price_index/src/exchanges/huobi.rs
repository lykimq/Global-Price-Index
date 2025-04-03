// REST client, polling logic
use crate::config::get_huobi_url;
use crate::error::{PriceIndexError, Result};
use crate::exchanges::Exchange;
use crate::models::{Order, OrderBook};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Huobi-specific implementation of the order book
///
/// Unlike other exchanges, Huobi returns price and quantity as float values directly,
/// so no custom deserializer is needed
#[derive(Debug, Serialize, Deserialize)]
struct HuobiOrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

/// The full response from Huobi API
///
/// Contains status code, error information, timestamp,
/// and the actual order book data in the "tick" field
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

/// HuobiExchange implements the Exchange trait for Huobi
///
/// This exchange uses REST API polling rather than WebSockets,
/// making periodic HTTP requests to fetch the current order book.
pub struct HuobiExchange {
    client: reqwest::Client,
}

impl HuobiExchange {
    /// Creates a new HuobiExchange instance
    ///
    /// This function:
    /// 1. Creates an HTTP client with a 5-second timeout
    /// 2. Verifies the exchange is accessible by making a test API request
    /// 3. Returns the exchange instance if successful
    ///
    /// Returns:
    ///   Result<Self>: The exchange instance or an error
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
    /// Returns the name of the exchange
    fn name(&self) -> &'static str {
        "Huobi"
    }

    /// Fetches the current order book from Huobi
    ///
    /// This function:
    /// 1. Makes an HTTP GET request to the Huobi API with appropriate parameters
    /// 2. Parses the JSON response into HuobiResponse
    /// 3. Converts the Huobi-specific format to our common OrderBook model
    ///
    /// Parameters:
    ///   - symbol: Trading pair (btcusdt)
    ///   - type: Depth type (step0 for highest precision)
    ///   - depth: Number of price levels (20)
    ///
    /// Returns:
    ///   Result<OrderBook>: The order book on success, or an error on failure
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
