use global_price_index::models::OrderBook;
use proptest::prelude::*;
use std::time::SystemTime;

proptest! {
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
}
