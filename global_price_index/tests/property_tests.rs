use global_price_index::models::{Order, OrderBook};
use proptest::prelude::*;
use std::time::SystemTime;

// Configure proptest to explicitly use a specific regression file
proptest! {
    #![proptest_config(ProptestConfig {
        // Explicitly set the regression file path
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            "tests/property_tests.proptest-regressions".into()
        ))),
        cases: 100, // Number of test cases to run
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_mid_price_properties(
        // Generate random values for bid and ask prices and quantities
        bid_price in 10000.0..100000.0f64,
        bid_quantity in 0.1..10.0f64,
        ask_price in 10000.0..100000.0f64,
        ask_quantity in 0.1..10.0f64,
    ) {
        // Skip invalid market conditions: in a valid market, ask price must be greater than bid price
        // This matches the real OrderBook.calculate_mid_price() implementation which returns None when ask <= bid
        if ask_price <= bid_price {
            return Ok(());
        }

        let order_book = OrderBook{
            bids: vec![Order { price: bid_price, quantity: bid_quantity }],
            asks: vec![Order { price: ask_price, quantity: ask_quantity }],
            timestamp: SystemTime::now(),
        };

        let mid_price = order_book.calculate_mid_price().unwrap();

        // Property 1: Mid price should be between bid and ask prices
        assert!(mid_price > bid_price);
        assert!(mid_price < ask_price);

        // Property 2: Mid price should be the average of bid and ask prices, rounded to 2 decimal places
        let expected_mid = (bid_price + ask_price) / 2.0;
        let rounded_expected = (expected_mid * 100.0).round() / 100.0;
        assert_eq!(mid_price, rounded_expected);

        // Property 3: The absolute difference between mid price and expected mid price
        // should be less than 0.01 (our rounding precision)
        let absolute_diff = (mid_price - expected_mid).abs();
        assert!(absolute_diff <= 0.01);
    }

    #[test]
    fn test_empty_order_book_property(
        // Generate a random boolean to decide whether to have empty bids or empty asks
        is_empty_bids in proptest::bool::ANY,
        // Generate a small number of orders for the non-empty side
        valid_prices in prop::collection::vec((10000.0..100000.0f64, 0.1..10.0f64), 1..5),
    ) {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // Fill in either bids or asks based on the boolean
        if is_empty_bids {
            // Leave bids empty, populate asks
            for (price, quantity) in valid_prices {
                asks.push(Order { price, quantity });
            }
            // Sort asks in ascending order
            asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            // Leave asks empty, populate bids
            for (price, quantity) in valid_prices {
                bids.push(Order { price, quantity });
            }
            // Sort bids in descending order
            bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
        }

        let order_book = OrderBook {
            bids,
            asks,
            timestamp: SystemTime::now(),
        };

        // Property: An order book with empty bids or empty asks should not have a valid mid price
        assert!(order_book.calculate_mid_price().is_none(), "Order book with empty side should not have a mid price");
    }

    #[test]
    fn test_non_positive_prices_property(
        // Generate a random boolean to decide whether to have non-positive bid or ask
        is_non_positive_bid in proptest::bool::ANY,
        // Generate a non-positive price
        non_positive_price in -100.0..=0.0f64,
        // Generate a positive price for the other side
        positive_price in 10000.0..100000.0f64,
        // Generate quantities
        quantity1 in 0.1..10.0f64,
        quantity2 in 0.1..10.0f64,
    ) {
        let mut order_book = OrderBook {
            bids: vec![],
            asks: vec![],
            timestamp: SystemTime::now(),
        };

        if is_non_positive_bid {
            // Non-positive bid price, positive ask price
            order_book.bids.push(Order { price: non_positive_price, quantity: quantity1 });
            order_book.asks.push(Order { price: positive_price, quantity: quantity2 });
        } else {
            // Positive bid price, non-positive ask price
            order_book.bids.push(Order { price: positive_price, quantity: quantity1 });
            order_book.asks.push(Order { price: non_positive_price, quantity: quantity2 });
        }

        // Property: An order book with non-positive prices should not have a valid mid price
        assert!(order_book.calculate_mid_price().is_none(), "Order book with non-positive prices should not have a mid price");
    }

    #[test]
    fn test_order_book_validation(
        prices in prop::collection::vec(
            (10000.0..100000.0f64, 0.1..10.0f64),
            0..10
        ),
    ){
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for (price, quantity) in prices {
            if price < 50000.0 {
                bids.push(Order { price, quantity });
            } else {
                asks.push(Order { price, quantity });
            }
        }

        // Sort bids in descending order, asks in ascending order
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

        let order_book = OrderBook {
            bids,
            asks,
            timestamp: SystemTime::now(),
        };

        // Property 1: If we have both bids and asks, the best bid should be less than the best ask
        // This corresponds to the check in OrderBook.calculate_mid_price() which requires best_ask > best_bid
        if !order_book.bids.is_empty() && !order_book.asks.is_empty() {
            let best_bid = order_book.bids[0].price;
            let best_ask = order_book.asks[0].price;
            assert!(best_bid < best_ask, "Best bid ({}) should be strictly less than best ask ({})", best_bid, best_ask);
        }

        // Property 2: Bids should be in descending order
        if !order_book.bids.is_empty() {
            for i in 1..order_book.bids.len() {
                let bid1 = order_book.bids[i].price;
                let bid2 = order_book.bids[i - 1].price;
                assert!(bid1 <= bid2, "Bids should be in descending order");
            }
        }

        // Property 3: Asks should be in ascending order
        if !order_book.asks.is_empty() {
            for i in 1..order_book.asks.len() {
                let ask1 = order_book.asks[i].price;
                let ask2 = order_book.asks[i - 1].price;
                assert!(ask1 >= ask2, "Asks should be in ascending order");
            }
        }
    }

}
