use global_price_index::models::OrderBook;
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
        // Ensure ask price is greater than bid price
        if ask_price <= bid_price {
            return Ok(());
        }

        let order_book = OrderBook{
            bids: vec![[bid_price.to_string(), bid_quantity.to_string()]],
            asks: vec![[ask_price.to_string(), ask_quantity.to_string()]],
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
                bids.push([price.to_string(), quantity.to_string()]);
            } else {
                asks.push([price.to_string(), quantity.to_string()]);
            }
        }

        // Sort bids in descending order, asks in ascending order
        bids.sort_by(|a, b| b[0].parse::<f64>().unwrap().partial_cmp(&a[0].parse::<f64>().unwrap()).unwrap());
        asks.sort_by(|a, b| a[0].parse::<f64>().unwrap().partial_cmp(&b[0].parse::<f64>().unwrap()).unwrap());

        let order_book = OrderBook {
            bids,
            asks,
            timestamp: SystemTime::now(),
        };

        // Property 1: If we have both bids and asks, the best bid should be
        // less than or equal to the best ask
        if !order_book.bids.is_empty() && !order_book.asks.is_empty() {
            let best_bid = order_book.bids[0][0].parse::<f64>().unwrap();
            let best_ask = order_book.asks[0][0].parse::<f64>().unwrap();
            assert!(best_bid <= best_ask);
        }

        // Property 2: Bids should be in descending order
        if !order_book.bids.is_empty() {
            for i in 1..order_book.bids.len() {
                let bid1 = order_book.bids[i][0].parse::<f64>().unwrap();
                let bid2 = order_book.bids[i - 1][0].parse::<f64>().unwrap();
                assert!(bid1 <= bid2);
            }
        }

        // Property 3: Asks should be in ascending order
        if !order_book.asks.is_empty() {
            for i in 1..order_book.asks.len() {
                let ask1 = order_book.asks[i][0].parse::<f64>().unwrap();
                let ask2 = order_book.asks[i - 1][0].parse::<f64>().unwrap();
                assert!(ask1 >= ask2);
            }
        }
    }

}
