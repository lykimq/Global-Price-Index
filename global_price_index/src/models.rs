// OrderBook, BidAsk, MidPrice
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<Order>, // [price, quantity]
    pub asks: Vec<Order>, // [price, quantity]
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
            .as_millis();
        serializer.serialize_i64(timestamp as i64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        let duration = std::time::Duration::from_millis(timestamp as u64);
        Ok(UNIX_EPOCH + duration)
    }
}

impl OrderBook {
    pub fn calculate_mid_price(&self) -> Option<f64> {
        if self.bids.is_empty() || self.asks.is_empty() {
            return None;
        }

        // Get the best bid (highest price) and best ask (lowest price)
        let best_bid = self.bids[0].price;
        if best_bid <= 0.0 {
            return None;
        }

        let best_ask = self.asks[0].price;
        if best_ask <= 0.0 {
            return None;
        }

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
        // Filter out invalid prices (keep only positive prices)
        let valid_exchanges: Vec<&ExchangePrice> = exchange_prices
            .iter()
            .filter(|ep| ep.mid_price > 0.0)
            .collect();

        let average_price = if !valid_exchanges.is_empty() {
            // Calculate weighted average based on timestamp recency
            let now = SystemTime::now();

            // -----------------------------------------------------------
            // Time-based weighting system
            // -----------------------------------------------------------
            // Rather than using a simple average where all prices have
            // equal influence, we apply time-based weighting to give
            // more recent prices higher influence on the final result.
            // This makes the global price more responsive to recent market changes.
            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;

            // The decay factor (in seconds) controls how quickly older prices lose influence
            // With a decay factor of 300 seconds (5 minutes):
            // - A price from right now gets weight = 1.0 (100% influence)
            // - A price 5 minutes old gets weight ≈ 0.368 (36.8% influence)
            // - A price 10 minutes old gets weight ≈ 0.135 (13.5% influence)
            // - A price 20 minutes old gets weight ≈ 0.018 (1.8% influence)
            let decay_factor = crate::config::get_decay_factor();

            for exchange_price in &valid_exchanges {
                // Calculate time difference between now and when the price was recorded
                // This tells us how "old" or "stale" this particular price data is
                let time_diff_secs = now
                    .duration_since(exchange_price.timestamp)
                    .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                    .as_secs() as f64;

                // Apply exponential decay formula: weight = e^(-time_diff/decay_factor)
                // This creates a smooth curve where:
                // - Recent prices get weights close to 1.0
                // - Older prices get weights approaching 0
                let weight = (-time_diff_secs / decay_factor).exp();

                // Add this price to our weighted sum
                weighted_sum += exchange_price.mid_price * weight;
                total_weight += weight;
            }

            // Calculate the final weighted average
            if total_weight > 0.0 {
                weighted_sum / total_weight
            } else {
                // Fallback to simple average if weighting fails
                // This should rarely happen but provides robustness
                valid_exchanges.iter().map(|ep| ep.mid_price).sum::<f64>()
                    / valid_exchanges.len() as f64
            }
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
