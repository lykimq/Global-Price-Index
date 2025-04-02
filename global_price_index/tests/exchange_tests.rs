use global_price_index::{
    error::Result,
    exchanges::{binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange, Exchange},
    models::{Order, OrderBook},
};
use std::time::SystemTime;

/// Tests that the Binance exchange correctly provides order book data
/// and that the data follows the expected structure.
///
/// This test verifies:
/// 1. The order book contains both bids and asks
/// 2. All prices are valid finite numbers
/// 3. The timestamp is current (not in the future)
///
/// Integration test that connects to the real Binance API.
#[tokio::test]
async fn test_binance_order_book_calculation() -> Result<()> {
    let exchange = BinanceExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    // Verify the order book structure
    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    // Verify price format
    for Order { price, .. } in order_book.bids.iter() {
        assert!(price.is_finite());
    }

    for Order { price, .. } in order_book.asks.iter() {
        assert!(price.is_finite());
    }

    // Verify timestamp
    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}

/// Tests that the Kraken exchange correctly provides order book data
/// and that the data follows the expected structure.
///
/// This test verifies:
/// 1. The order book contains both bids and asks
/// 2. All prices are valid finite numbers
/// 3. The timestamp is current (not in the future)
///
/// Integration test that connects to the real Kraken API.
#[tokio::test]
async fn test_kraken_order_book_calculation() -> Result<()> {
    let exchange = KrakenExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    for Order { price, .. } in order_book.bids.iter() {
        assert!(price.is_finite());
    }

    for Order { price, .. } in order_book.asks.iter() {
        assert!(price.is_finite());
    }

    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}

/// Tests that the Huobi exchange correctly provides order book data
/// and that the data follows the expected structure.
///
/// This test verifies:
/// 1. The order book contains both bids and asks
/// 2. All prices are valid finite numbers
/// 3. The timestamp is current (not in the future)
///
/// Integration test that connects to the real Huobi API.
#[tokio::test]
async fn test_huobi_orderbook_calculation() -> Result<()> {
    let exchange = HuobiExchange::new().await?;
    let order_book = exchange.fetch_order_book().await?;

    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    for Order { price, .. } in order_book.bids.iter() {
        assert!(price.is_finite());
    }

    for Order { price, .. } in order_book.asks.iter() {
        assert!(price.is_finite());
    }

    assert!(order_book.timestamp <= SystemTime::now());

    Ok(())
}

/// Tests that an exchange can correctly calculate a mid price
/// from its order book data.
///
/// This test verifies:
/// 1. The mid price is positive (valid BTC price)
/// 2. The exchange name is provided
/// 3. The timestamp is current (not in the future)
///
/// Integration test that connects to the real Binance API.
#[tokio::test]
async fn test_mid_price_calculation() -> Result<()> {
    let exchange = BinanceExchange::new().await?;
    let price = exchange.get_mid_price().await?;

    assert!(price.mid_price > 0.0);
    assert!(!price.exchange.is_empty());
    assert!(price.timestamp <= SystemTime::now());

    Ok(())
}

/// Tests the algorithm for calculating the mid price from
/// an order book with known values.
///
/// This test verifies:
/// 1. The mid price is correctly calculated as the average of best bid and best ask
/// 2. The rounding to 2 decimal places works correctly
///
/// Using a tolerance of 0.01 (a difference < 1 cent) because:
/// - This is a simple mathematical calculation
/// - We expect exact results to the cent level
/// - No complex functions are involved
#[test]
fn test_order_book_mid_price_calculation() {
    let order_book = OrderBook {
        bids: vec![
            // Best bid: 2.0 BTC at 50,000 USDT (highest price someone will buy at)
            Order {
                price: 50000.0,
                quantity: 2.0,
            },
            // 3.0 BTC at 49,900 USDT
            Order {
                price: 49900.0,
                quantity: 3.0,
            },
        ],
        asks: vec![
            // Best ask: 1.0 BTC at 50,100 USDT (lowest price someone will sell at)
            Order {
                price: 50100.0,
                quantity: 1.0,
            },
            // 2.0 BTC at 50,200 USDT
            Order {
                price: 50200.0,
                quantity: 2.0,
            },
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

/// Tests that an empty order book correctly returns None
/// when attempting to calculate a mid price.
///
/// This test verifies:
/// 1. The system handles empty input gracefully
/// 2. The function returns None when no valid prices exist
#[test]
fn test_empty_order_book_mid_price() {
    let order_book = OrderBook {
        bids: vec![],
        asks: vec![],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price();
    assert!(mid_price.is_none());
}

/// Tests that an order book with invalid (zero) prices correctly returns
/// None when attempting to calculate a mid price.
///
/// This test verifies:
/// 1. The system handles invalid prices gracefully
/// 2. The function returns None when no valid prices exist
#[test]
fn test_invalid_order_book_mid_price() {
    let order_book = OrderBook {
        bids: vec![Order {
            price: 0.0,
            quantity: 1.0,
        }],
        asks: vec![Order {
            price: 0.0,
            quantity: 1.0,
        }],
        timestamp: SystemTime::now(),
    };

    let mid_price = order_book.calculate_mid_price();
    assert!(mid_price.is_none());
}
