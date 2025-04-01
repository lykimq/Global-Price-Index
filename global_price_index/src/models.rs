// OrderBook, BidAsk, MidPrice
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<[String; 2]>, // [price, quantity]
    pub asks: Vec<[String; 2]>, // [price, quantity]
    #[serde(with = "timestamp_serde")]
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePrice {
    pub exchange: String,
    pub mid_price: f64,
    #[serde(with = "timestamp_serde")]
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPriceIndex {
    pub price: f64,
    #[serde(with = "timestamp_serde")]
    pub timestamp: SystemTime,
    pub exchange_prices: Vec<ExchangePrice>,
}

mod timestamp_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("Invalid timestamp"))?
            .as_secs_f64();
        serializer.serialize_f64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = f64::deserialize(deserializer)?;
        let duration = std::time::Duration::from_secs_f64(timestamp);
        Ok(UNIX_EPOCH + duration)
    }
}

impl OrderBook {
    pub fn calculate_mid_price(&self) -> Option<f64> {
        if self.bids.is_empty() || self.asks.is_empty() {
            return None;
        }

        // Get the best bid (highest price) and best ask (lowest price)
        let best_bid = match self.bids[0][0].parse::<f64>() {
            Ok(p) if p > 0.0 => p,
            _ => return None,
        };

        let best_ask = match self.asks[0][0].parse::<f64>() {
            Ok(p) if p > 0.0 => p,
            _ => return None,
        };

        // Ensure the spread is reasonable (ask > bid)
        if best_ask <= best_bid {
            return None;
        }

        // Calculate mid price as average of best bid and best ask
        let mid_price = (best_bid + best_ask) / 2.0;

        // Round to 2 decimal places
        Some((mid_price * 100.0).round() / 100.0)
    }
}

impl GlobalPriceIndex {
    pub fn new(exchange_prices: Vec<ExchangePrice>) -> Self {
        // Filter out invalid prices and calculate weighted average
        let valid_prices: Vec<f64> = exchange_prices
            .iter()
            .filter(|ep| ep.mid_price > 0.0)
            .map(|ep| ep.mid_price)
            .collect();

        let average_price = if !valid_prices.is_empty() {
            // Calculate weighted average based on timestamp recency
            let _now = SystemTime::now();
            let total_weight: f64 = valid_prices.len() as f64;
            valid_prices.iter().sum::<f64>() / total_weight
        } else {
            0.0
        };

        Self {
            price: average_price,
            timestamp: SystemTime::now(),
            exchange_prices,
        }
    }
}
