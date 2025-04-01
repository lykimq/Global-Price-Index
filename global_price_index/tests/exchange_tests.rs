use global_price_index::{
    exchanges::{binance::BinanceExchange, Exchange},
    error::Result
};
use std::time::SystemTime;

#[tokio::test]
async fn test_binance_order_book_calculation() -> Result<()> {
    let exchange = BinanceExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    // Verify the order book structure
    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    // Verify price format
    for [price, _] in order_book.bids.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    for [price, _] in order_book.asks.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    // Verify timestamp
    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}