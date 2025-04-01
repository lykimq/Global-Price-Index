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
            // 2.0 BTC at 50,000 USDT each = 100,000 USDT worth
            ["50000.0".to_string(), "2.0".to_string()],
            // 3.0 BTC at 49,900 USDT each = 149,700 USDT worth
            ["49900.0".to_string(), "3.0".to_string()],
        ],
        asks: vec![
            // 1.0 BTC at 50,100 USDT each = 50,100 USDT worth
            ["50100.0".to_string(), "1.0".to_string()],
            // 2.0 BTC at 50,200 USDT each = 100,400 USDT worth
            ["50200.0".to_string(), "2.0".to_string()],
        ],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price().unwrap();

    // Mid price calculation:
    // Best bid: 50000.0
    // Best ask: 50100.0
    // Mid price: (50000.0 + 50100.0) / 2 = 50050.0
    assert!((mid_price - 50050.0).abs() < 0.01);
}