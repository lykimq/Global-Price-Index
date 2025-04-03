// Exchange trait, factory
use crate::error::{PriceIndexError, Result};
use crate::models::{ExchangePrice, OrderBook};
use async_trait::async_trait;
use std::time::SystemTime;

pub mod binance;
pub mod huobi;
pub mod kraken;

/// The Exchange trait defines the interface for cryptocurrency exchanges.
///
/// All exchange implementations must implement this trait to provide a
/// consistent interface for fetching order book data and calculating
/// mid-prices, regardless of the exchange-specific API details.
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Returns the name of the exchange as a static string
    fn name(&self) -> &'static str;

    /// Fetches the current order book from the exchange
    ///
    /// This method must be implemented by each exchange to handle the
    /// specific API details for retrieving order book data.
    ///
    /// Returns:
    ///   Result<OrderBook>: The order book on success, or an error on failure
    async fn fetch_order_book(&self) -> Result<OrderBook>;

    /// Calculates the mid-price from the exchange's order book
    ///
    /// This is a default implementation that:
    /// 1. Fetches the order book using fetch_order_book()
    /// 2. Calculates the mid-price using OrderBook::calculate_mid_price()
    /// 3. Returns an ExchangePrice with the exchange name, mid-price, and current timestamp
    ///
    /// This method can be overridden by exchanges if they have a more efficient
    /// way to get mid-prices directly.
    ///
    /// Returns:
    ///   Result<ExchangePrice>: The exchange price on success, or an error on failure
    async fn get_mid_price(&self) -> Result<ExchangePrice> {
        let order_book = self.fetch_order_book().await?;
        let mid_price = order_book.calculate_mid_price().ok_or_else(|| {
            PriceIndexError::InvalidPriceData(format!(
                "Failed to calculate mid price for {}",
                self.name()
            ))
        })?;

        Ok(ExchangePrice {
            exchange: self.name().to_string(),
            mid_price,
            timestamp: SystemTime::now(),
        })
    }
}
