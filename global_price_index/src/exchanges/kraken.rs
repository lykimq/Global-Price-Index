// REST client, polling logic

use crate::config::get_kraken_url;
use crate::error::{PriceIndexError, Result};
use crate::exchanges::Exchange;
use crate::models::{Order, OrderBook};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Kraken-specific implementation of the order book
/// Contains bids and asks in the format returned by Kraken API
#[derive(Debug, Serialize, Deserialize)]
struct KrakenOrderBook {
    #[serde(deserialize_with = "deserialize_kraken_orders")]
    bids: Vec<Order>,
    #[serde(deserialize_with = "deserialize_kraken_orders")]
    asks: Vec<Order>,
}

/// Custom deserializer for Kraken order data format
///
/// Kraken returns orders as [price: String, volume: String, timestamp: Integer (Unix time)]
/// This function converts them to our Order struct with f64 values for price and quantity
fn deserialize_kraken_orders<'de, D>(deserializer: D) -> std::result::Result<Vec<Order>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let raw: Vec<[serde_json::Value; 3]> = Vec::deserialize(deserializer)?;

    raw.into_iter()
        .map(|[price, volume, _timestamp]| {
            let price_str = price
                .as_str()
                .ok_or_else(|| D::Error::custom("price must be a string"))?;
            let volume_str = volume
                .as_str()
                .ok_or_else(|| D::Error::custom("volume must be a string"))?;

            let price = price_str
                .parse::<f64>()
                .map_err(|_| D::Error::custom("Failed to parse price as f64"))?;
            let quantity = volume_str
                .parse::<f64>()
                .map_err(|_| D::Error::custom("Failed to parse volume as f64"))?;

            Ok(Order { price, quantity })
        })
        .collect()
}

/// Represents the result field from Kraken API response
/// Contains the order book data for XBTUSDT trading pair
#[derive(Debug, Serialize, Deserialize)]
pub struct KrakenResult {
    #[serde(rename = "XBTUSDT")]
    xbtusdt: KrakenOrderBook,
}

/// The full response from Kraken API
/// Contains an error field and the result data
#[derive(Debug, Serialize, Deserialize)]
struct KrakenResponse {
    error: Vec<String>,
    result: KrakenResult,
}

/// KrakenExchange implements the Exchange trait for Kraken
///
/// This exchange uses REST API polling rather than WebSockets,
/// making periodic HTTP requests to fetch the current order book.
pub struct KrakenExchange {
    client: reqwest::Client,
}

impl KrakenExchange {
    /// Creates a new KrakenExchange instance
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
        let params = [("pair", "XBTUSDT"), ("count", "1")];
        let response: KrakenResponse = client
            .get(&get_kraken_url())
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        if !response.error.is_empty() {
            return Err(PriceIndexError::ExchangeError(format!(
                "Kraken API error during initialization: {:?}",
                response.error
            )));
        }

        Ok(Self { client })
    }
}

#[async_trait]
impl Exchange for KrakenExchange {
    /// Returns the name of the exchange
    fn name(&self) -> &'static str {
        "Kraken"
    }

    /// Fetches the current order book from Kraken
    ///
    /// This function:
    /// 1. Makes an HTTP GET request to the Kraken API
    /// 2. Parses the JSON response into KrakenResponse
    /// 3. Converts the Kraken-specific format to our common OrderBook model
    ///
    /// Returns:
    ///   Result<OrderBook>: The order book on success, or an error on failure
    async fn fetch_order_book(&self) -> Result<OrderBook> {
        let params = [("pair", "XBTUSDT"), ("count", "100")];
        let response: KrakenResponse = self
            .client
            .get(&get_kraken_url())
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        if !response.error.is_empty() {
            return Err(PriceIndexError::ExchangeError(format!(
                "Kraken API error: {:?}",
                response.error
            )));
        }

        let order_book = response.result.xbtusdt;
        Ok(OrderBook {
            bids: order_book.bids,
            asks: order_book.asks,
            timestamp: SystemTime::now(),
        })
    }
}
