// REST client, polling logic

use crate::exchanges::Exchange;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{PriceIndexError, Result};
use crate::models::OrderBook;
use std::time::SystemTime;
use dotenv::dotenv;
use std::env;

// Load environment variable with fallback
fn get_kraken_url() -> String {
    dotenv().ok();
    env::var("KRAKEN_URL")
        .unwrap_or_else(|_| "https://api.kraken.com/0/public/Depth?pair=XBTUSDT".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct KrakenOrderBook {
    #[serde(deserialize_with = "deserialize_order_book")]
    bids: Vec<[String; 3]>, // [price, volume, timestamp]
    #[serde(deserialize_with = "deserialize_order_book")]
    asks: Vec<[String; 3]>, // [price, volume, timestamp]
}

fn deserialize_order_book<'de, D>(deserializer: D) -> std::result::Result<Vec<[String; 3]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let raw: Vec<[serde_json::Value; 3]> = Vec::deserialize(deserializer)?;

    raw.into_iter()
        .map(|[price, volume, timestamp]| {
            let price = price.as_str().ok_or_else(|| D::Error::custom("price must be a string"))?.to_string();
            let volume = volume.as_str().ok_or_else(|| D::Error::custom("volume must be a string"))?.to_string();
            let timestamp = timestamp.as_i64()
                .ok_or_else(|| D::Error::custom("timestamp must be a number"))?
                .to_string();

            Ok([price, volume, timestamp])
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
            .map_err(|e| PriceIndexError::ExchangeError(format!("Failed to create HTTP client: {}", e)))?;

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
            bids: order_book.bids.into_iter().map(|[price, _, _]| [price, "0".to_string()]).collect(),
            asks: order_book.asks.into_iter().map(|[price, _, _]| [price, "0".to_string()]).collect(),
            timestamp: SystemTime::now(),
        })
    }
}
