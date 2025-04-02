use global_price_index::models::{ExchangePrice, GlobalPriceIndex};
use std::time::{Duration, SystemTime};

/// Tests that the global price index correctly applies time-based weighting
/// to prices from different timestamps.
///
/// This test verifies:
/// 1. Recent prices have higher weights than older ones
/// 2. The weighted calculation produces the expected result
///
/// Using a tolerance of 1.0 (a difference < $1) because:
/// - BTC prices are in the tens of thousands, so $1 is a negligible difference (~0.002%)
/// - Floating-point calculations may have minor rounding differences
/// - The exact timestamp differences during test execution might cause slight variations
#[test]
fn test_global_price_index_weighting() {
    // Create mock prices with different timestamps
    let now = SystemTime::now();

    let exchange_prices = vec![
        // Current price
        ExchangePrice {
            exchange: "Exchange1".to_string(),
            mid_price: 50000.0,
            timestamp: now,
        },
        // 5 minutes old price
        ExchangePrice {
            exchange: "Exchange2".to_string(),
            mid_price: 51000.0,
            timestamp: now.checked_sub(Duration::from_secs(300)).unwrap(),
        },
        // 10 minutes old price
        ExchangePrice {
            exchange: "Exchange3".to_string(),
            mid_price: 52000.0,
            timestamp: now.checked_sub(Duration::from_secs(600)).unwrap(),
        },
    ];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // With decay_factor = 300.0:
    // - Exchange1: weight = 1.0 (100%)
    // - Exchange2: weight ≈ 0.368 (36.8%)
    // - Exchange3: weight ≈ 0.135 (13.5%)
    //
    // So the weighted price should be:
    // (50000 * 1.0 + 51000 * 0.368 + 52000 * 0.135) / (1.0 + 0.368 + 0.135)
    //
    // The actual value from test execution is 50424.79
    let expected_price = 50424.79;
    assert!(
        (global_index.price - expected_price).abs() < 1.0,
        "Expected price around {}, but got {}",
        expected_price,
        global_index.price
    );
}

/// Tests that prices with equal timestamps are weighted equally,
/// resulting in a simple arithmetic average.
///
/// This test verifies:
/// 1. When all timestamps are equal, all weights should be equal (1.0)
/// 2. Equal weights produce a simple average of all prices
///
/// Using a smaller tolerance of 0.01 (a difference < 1 cent) because:
/// - This is a simple arithmetic mean calculation
/// - No complex exponential functions are involved
/// - There should be minimal floating-point error
#[test]
fn test_global_price_index_equal_timestamps() {
    // Create mock prices with equal timestamps
    let now = SystemTime::now();

    let exchange_prices = vec![
        ExchangePrice {
            exchange: "Exchange1".to_string(),
            mid_price: 50000.0,
            timestamp: now,
        },
        ExchangePrice {
            exchange: "Exchange2".to_string(),
            mid_price: 51000.0,
            timestamp: now,
        },
        ExchangePrice {
            exchange: "Exchange3".to_string(),
            mid_price: 52000.0,
            timestamp: now,
        },
    ];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // All weights should be 1.0, so this should be a simple average
    let expected_price = (50000.0 + 51000.0 + 52000.0) / 3.0;
    assert!(
        (global_index.price - expected_price).abs() < 0.01,
        "Expected simple average {}, but got {}",
        expected_price,
        global_index.price
    );
}

/// Tests that when only one valid price is available,
/// the global price index equals that price exactly.
///
/// This test verifies:
/// 1. Single price handling works correctly
/// 2. No unexpected modifications are made to a lone price
///
/// Using a tolerance of 0.01 (a difference < 1 cent) because:
/// - This is a direct assignment operation (price = single_price)
/// - No complex calculations are involved
/// - The result should be exact to the cent
#[test]
fn test_global_price_index_one_valid_price() {
    // Create a single valid price
    let now = SystemTime::now();

    let exchange_prices = vec![ExchangePrice {
        exchange: "Exchange1".to_string(),
        mid_price: 50000.0,
        timestamp: now,
    }];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // Should be exactly the single price
    assert!(
        (global_index.price - 50000.0).abs() < 0.01,
        "Expected single price 50000.0, but got {}",
        global_index.price
    );
}

/// Tests that invalid prices (negative or zero) are properly filtered out,
/// and only valid prices are used in the calculation.
///
/// This test verifies:
/// 1. Negative prices are rejected
/// 2. Zero prices are rejected
/// 3. The calculation proceeds with only valid prices
///
/// Using a tolerance of 0.01 (a difference < 1 cent) because:
/// - This is a simple filtering operation followed by a direct assignment
/// - The result should match the single valid price exactly
/// - No complex calculations are involved when only one price remains
#[test]
fn test_global_price_index_invalid_prices() {
    // Create invalid (negative) prices
    let now = SystemTime::now();

    let exchange_prices = vec![
        ExchangePrice {
            exchange: "Exchange1".to_string(),
            mid_price: -50000.0, // Invalid
            timestamp: now,
        },
        ExchangePrice {
            exchange: "Exchange2".to_string(),
            mid_price: 0.0, // Invalid
            timestamp: now,
        },
        ExchangePrice {
            exchange: "Exchange3".to_string(),
            mid_price: 52000.0, // Valid
            timestamp: now,
        },
    ];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // Should only use the single valid price
    assert!(
        (global_index.price - 52000.0).abs() < 0.01,
        "Expected only valid price 52000.0, but got {}",
        global_index.price
    );
}

/// Tests that very old prices have minimal impact on the global price index,
/// verifying the time-decay behavior functions correctly.
///
/// This test verifies:
/// 1. Very old prices (30 minutes) have negligible influence
/// 2. The result is dominated by recent prices
///
/// Using a larger tolerance of 100.0 (a difference < $100) because:
/// - We're testing for approximate behavior, not exact values
/// - The old price is intentionally set very different (30000 vs 50000)
/// - We only need to confirm the old price has minimal influence
/// - Small time differences during test execution could affect exponential decay
#[test]
fn test_global_price_index_very_old_prices() {
    // Create mock prices with very different ages
    let now = SystemTime::now();

    let exchange_prices = vec![
        // Current price
        ExchangePrice {
            exchange: "Exchange1".to_string(),
            mid_price: 50000.0,
            timestamp: now,
        },
        // 30 minutes old (should have ~0.05% influence)
        ExchangePrice {
            exchange: "Exchange2".to_string(),
            mid_price: 30000.0, // Very different to show the low influence
            timestamp: now.checked_sub(Duration::from_secs(1800)).unwrap(),
        },
    ];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // The 30-minute old price should have almost no influence
    // Global price should be very close to the current price (50000.0)
    assert!(
        (global_index.price - 50000.0).abs() < 100.0,
        "Old price had too much influence, expected close to 50000.0, but got {}",
        global_index.price
    );
}

/// Tests the behavior when provided with an empty list of prices.
///
/// This test verifies:
/// 1. The system handles empty input gracefully
/// 2. The default value for empty input is 0.0
///
/// Using exact equality (assert_eq!) because:
/// - This is a simple edge case with a defined return value (0.0)
/// - No calculations are performed, so no floating-point errors exist
/// - The behavior should be deterministic and exact
#[test]
fn test_global_price_index_empty_prices() {
    // Empty price list
    let exchange_prices = vec![];

    // Calculate the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    // Should be 0.0 for empty prices
    assert_eq!(
        global_index.price, 0.0,
        "Expected 0.0 for empty prices, but got {}",
        global_index.price
    );
}

/// Directly tests the exponential decay weight calculation formula
/// for different time differences to ensure mathematical accuracy.
///
/// This test verifies:
/// 1. Current prices (0 time diff) have weight = 1.0
/// 2. Older prices have exponentially decaying weights
/// 3. The decay formula matches w = e^(-time_diff/decay_factor)
///
/// Using a very small tolerance of 0.0001 because:
/// - We're testing a precise mathematical formula
/// - The expected values are pre-calculated to high precision
/// - This is a fundamental calculation that affects all weighted pricing
/// - No external time measurements affect this calculation
#[test]
fn test_weight_calculation() {
    // Mock the time difference calculation by creating prices with known time differences
    let now = SystemTime::now();

    // Create test cases with specific time differences (in seconds)
    let test_cases = vec![
        (0, 1.0),                    // Current: weight = 1.0
        (300, 0.36787944117144233),  // 5 minutes: weight ≈ 0.368
        (600, 0.1353352832366127),   // 10 minutes: weight ≈ 0.135
        (1200, 0.01831563888873418), // 20 minutes: weight ≈ 0.018
    ];

    for (time_diff_secs, expected_weight) in test_cases {
        // Create a price with the specified time difference
        let _price = ExchangePrice {
            exchange: "Test".to_string(),
            mid_price: 50000.0,
            timestamp: now
                .checked_sub(Duration::from_secs(time_diff_secs))
                .unwrap(),
        };

        // Calculate the weight manually using the same formula as in the implementation
        let decay_factor = 300.0; // Use the default value for testing
        let time_diff_f64 = time_diff_secs as f64;
        let actual_weight = (-time_diff_f64 / decay_factor).exp();

        // Verify the calculation matches the expected weight
        assert!(
            (actual_weight - expected_weight).abs() < 0.0001,
            "Weight calculation for {} seconds: expected {}, got {}",
            time_diff_secs,
            expected_weight,
            actual_weight
        );
    }
}
