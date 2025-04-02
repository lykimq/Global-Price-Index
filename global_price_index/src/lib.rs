//! Global BTC/USDT Price Index API
//!
//! This library provides functionality for aggregating and serving real-time BTC/USDT price data
//! from multiple cryptocurrency exchanges.

pub mod api;
pub mod config;
pub mod error;
pub mod exchanges;
pub mod models;

// Re-export commonly used items
pub use api::start_server;
pub use config::SETTINGS;
pub use error::{PriceIndexError, Result};
pub use models::{ExchangePrice, GlobalPriceIndex, OrderBook};

// Re-export exchange types
pub use exchanges::binance::BinanceExchange;
pub use exchanges::huobi::HuobiExchange;
pub use exchanges::kraken::KrakenExchange;
pub use exchanges::Exchange;
