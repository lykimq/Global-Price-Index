// Exchange trait, factory
use crate::error::{PriceIndexError, Result};
use crate::models::{ExchangePrice, OrderBook};
use async_trait::async_trait;
use std::time::SystemTime;

pub mod binance;
pub mod huobi;
pub mod kraken;

#[async_trait]
pub trait Exchange: Send + Sync {
    fn name(&self) -> &'static str;

    async fn fetch_order_book(&self) -> Result<OrderBook>;

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
