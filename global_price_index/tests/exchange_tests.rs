use global_price_index::{
    exchanges::{binance::BinanceExchange, kraken::KrakenExchange, huobi::HuobiExchange, Exchange},
    models::OrderBook,
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

#[tokio::test]
async fn test_kraken_order_book_calculation() -> Result<()>{
    let exchange = KrakenExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    for [price, _] in order_book.bids.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    for [price, _] in order_book.asks.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}

#[tokio::test]
async fn test_huobi_orderbook_calculation() -> Result<()> {
    let exchange = HuobiExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    for [price, _] in order_book.bids.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    for [price, _] in order_book.asks.iter() {
        assert!(price.parse::<f64>().is_ok());
    }

    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}

#[tokio::test]
async fn test_mid_price_calculation() -> Result<()>{
    let exchange = BinanceExchange::new().await?;
    let price = exchange.get_mid_price().await?;

    assert!(price.mid_price > 0.0);
    assert!(!price.exchange.is_empty());
    assert!(price.timestamp <= SystemTime::now());

    Ok(())
}

#[test]
fn test_order_book_mid_price_calculation() {
    let order_book = OrderBook {
        bids: vec![
            // Best bid: 2.0 BTC at 50,000 USDT (highest price someone will buy at)
            ["50000.0".to_string(), "2.0".to_string()],
            // 3.0 BTC at 49,900 USDT
            ["49900.0".to_string(), "3.0".to_string()],
        ],
        asks: vec![
            // Best ask: 1.0 BTC at 50,100 USDT (lowest price someone will sell at)
            ["50100.0".to_string(), "1.0".to_string()],
            // 2.0 BTC at 50,200 USDT
            ["50200.0".to_string(), "2.0".to_string()],
        ],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price().unwrap();

    // Mid price calculation:
    // Best bid: 50000.0 (highest buy price)
    // Best ask: 50100.0 (lowest sell price)
    // Mid price: (50000.0 + 50100.0) / 2 = 50050.0
    // Implementation rounds to 2 decimal places
    assert!((mid_price - 50050.0).abs() < 0.01);
}

#[test]
fn test_empty_order_book_mid_price() {
    let order_book = OrderBook{
        bids: vec![],
        asks: vec![],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price();
    assert!(mid_price.is_none());
}

#[test]
fn test_invalid_order_book_mid_price() {
    let order_book = OrderBook {
        bids: vec![["invalid".to_string(), "1.0".to_string()]],
        asks: vec![["50100.0".to_string(), "1.0".to_string()]],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price();
    assert!(mid_price.is_none());
}