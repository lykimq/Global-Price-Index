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
fn get_kraken_url() -> String {
    dotenv().ok();
    env::var("KRAKEN_URL")
        .unwrap_or_else(|_| "https://api.kraken.com/0/public/Depth?pair=XBTUSDT".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct KrakenOrderBook {
    #[serde(deserialize_with = "deserialize_kraken_orders")]
    bids: Vec<Order>,
    #[serde(deserialize_with = "deserialize_kraken_orders")]
    asks: Vec<Order>,
}

// Kraken returns [price: String, volume: String, timestamp: Integer (Unix time)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct KrakenResult {
    #[serde(rename = "XBTUSDT")]
    xbtusdt: KrakenOrderBook,
}

#[derive(Debug, Serialize, Deserialize)]
struct KrakenResponse {
    error: Vec<String>,
    result: KrakenResult,
}

pub struct KrakenExchange {
    client: reqwest::Client,
}

impl KrakenExchange {
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
    fn name(&self) -> &'static str {
        "Kraken"
    }

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
