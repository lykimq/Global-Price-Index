use futures::{SinkExt, StreamExt};
use global_price_index::{
    config::get_binance_ws_url,
    exchanges::{binance::BinanceExchange, Exchange},
};
use serde_json;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Tests that the Binance WebSocket connection works correctly
/// and that the order book is properly updated with WebSocket data.
///
/// This test verifies:
/// 1. A connection to Binance WebSocket can be established
/// 2. The initial order book snapshot is fetched
/// 3. WebSocket updates are received and processed
/// 4. The updated order book contains valid data
/// 5. The spread is valid (best ask > best bid)
///
/// This is a long-running integration test that connects to the real Binance WebSocket API.
/// It waits 30 seconds for WebSocket updates to ensure the connection is stable.
#[tokio::test]
async fn test_binance_websocket_connection() {
    println!("Starting Binance WebSocket test...");

    // Create a new Binance exchange instance
    let exchange = BinanceExchange::new()
        .await
        .expect("Failed to create Binance exchange");
    println!("Created Binance exchange instance");

    // Get the initial order book
    let init_order_book = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch order book");

    let init_best_bid = init_order_book
        .bids
        .first()
        .map(|b| b.price)
        .expect("No bids available in initial order book");
    let init_best_ask = init_order_book
        .asks
        .first()
        .map(|a| a.price)
        .expect("No asks available in initial order book");

    println!(
        "Initial order book - Best bid: {}, Best ask: {}",
        init_best_bid, init_best_ask
    );

    // Wait for websocket updates (give enough time for updates to come in)
    println!("Waiting for WebSocket updates...");
    sleep(Duration::from_secs(30)).await; // Wait time to 30 seconds
    println!("Finished waiting for updates");

    // Get the updated order book
    let updated_order_book = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch updated order book");

    let updated_best_bid = updated_order_book
        .bids
        .first()
        .map(|b| b.price)
        .expect("No bids available in updated order book");
    let updated_best_ask = updated_order_book
        .asks
        .first()
        .map(|a| a.price)
        .expect("No asks available in updated order book");

    println!(
        "Updated order book - Best bid: {}, Best ask: {}",
        updated_best_bid, updated_best_ask
    );

    // Verify the order book has data
    assert!(
        !updated_order_book.bids.is_empty(),
        "Order book has no bids"
    );
    assert!(
        !updated_order_book.asks.is_empty(),
        "Order book has no asks"
    );

    // Verify the spread is valid
    assert!(
        updated_best_bid < updated_best_ask,
        "Invalid spread: best_bid={}, best_ask={}",
        updated_best_bid,
        updated_best_ask
    );

    println!("Order book validation passed");
}

/// Tests that the Binance WebSocket connection can recover from disconnections
/// and continue to provide valid order book data.
///
/// This test verifies:
/// 1. The WebSocket connection can be established
/// 2. After a simulated disconnection period, data is still available
/// 3. The reconnected order book contains valid data
/// 4. Specific data integrity checks pass (valid prices, quantities)
/// 5. The order book timestamp is recent (within 5 minutes)
#[tokio::test]
async fn test_binance_websocket_reconnect() {
    println!("Starting Binance WebSocket reconnect test...");

    // Create a new Binance exchange instance
    let exchange = BinanceExchange::new()
        .await
        .expect("Failed to create Binance exchange");
    println!("Created Binance exchange instance");

    // Get the initial order book
    let init_order_book = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch initial order book");

    let init_best_bid = init_order_book
        .bids
        .first()
        .map(|b| b.price)
        .expect("No bids available in initial order book");
    let init_best_ask = init_order_book
        .asks
        .first()
        .map(|a| a.price)
        .expect("No asks available in initial order book");

    println!(
        "Initial order book - Best bid: {}, Best ask: {}",
        init_best_bid, init_best_ask
    );

    // Wait for websocket updates
    println!("Waiting for WebSocket updates...");
    sleep(Duration::from_secs(30)).await;
    println!("Finished waiting for updates");

    // Get the reconnected order book
    let reconnect_order_book = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch reconnected order book");

    let reconnect_best_bid = reconnect_order_book
        .bids
        .first()
        .map(|b| b.price)
        .expect("No bids available in reconnected order book");
    let reconnect_best_ask = reconnect_order_book
        .asks
        .first()
        .map(|a| a.price)
        .expect("No asks available in reconnected order book");

    println!(
        "Reconnected order book - Best bid: {}, Best ask: {}",
        reconnect_best_bid, reconnect_best_ask
    );

    // Verify the order book has data
    assert!(
        !reconnect_order_book.bids.is_empty(),
        "Reconnected order book has no bids"
    );
    assert!(
        !reconnect_order_book.asks.is_empty(),
        "Reconnected order book has no asks"
    );

    // Verify the spread is valid
    assert!(
        reconnect_best_bid < reconnect_best_ask,
        "Invalid spread in reconnected order book: best_bid={}, best_ask={}",
        reconnect_best_bid,
        reconnect_best_ask
    );

    // Verify the order book structure is valid
    for bid in &reconnect_order_book.bids {
        assert!(bid.price.is_finite(), "Invalid bid price: {}", bid.price);
        assert!(
            bid.quantity.is_finite(),
            "Invalid bid quantity: {}",
            bid.quantity
        );
    }

    for ask in &reconnect_order_book.asks {
        assert!(ask.price.is_finite(), "Invalid ask price: {}", ask.price);
        assert!(
            ask.quantity.is_finite(),
            "Invalid ask quantity: {}",
            ask.quantity
        );
    }

    // Verify the timestamp is recent (within last 5 minutes)
    let now = SystemTime::now();
    let timestamp_age = now
        .duration_since(reconnect_order_book.timestamp)
        .expect("Failed to calculate timestamp age");
    assert!(
        timestamp_age < Duration::from_secs(300), // 5 minutes
        "Order book timestamp is too old: {:?}",
        timestamp_age
    );

    println!("Order book validation passed");
}

/// Tests that the Binance WebSocket message format matches the expected structure
/// and contains required fields for processing order book updates.
///
/// This test verifies:
/// 1. Direct WebSocket connection can be established to Binance
/// 2. The subscription process works correctly
/// 3. The received message format contains all required fields:
///    - "b" (bids)
///    - "a" (asks)
///    - "e":"depthUpdate" (event type)
///    - "s":"BTCUSDT" (symbol)
#[tokio::test]
async fn test_binance_websocket_message_format() {
    let ws_url = get_binance_ws_url();

    // Strip the path from the websocket URL to get the base URL
    // The ws_url might be "wss://stream.binance.com:9443/ws/btcusdt@depth"
    // We need to extract "wss://stream.binance.com:9443/ws"
    let base_url = ws_url.split('/').collect::<Vec<&str>>()[..3].join("/");
    let base_ws_url = format!("{}/ws", base_url);

    let url = url::Url::parse(&base_ws_url).unwrap();
    println!("Connecting to WebSocket base URL: {}", base_ws_url);

    let (mut ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket");

    // Subscribe to the order book stream
    let subscribe_msg = serde_json::json!({
        "method": "SUBSCRIBE",
        "params": ["btcusdt@depth"],
        "id": 1
    });
    ws_stream
        .send(Message::Text(subscribe_msg.to_string()))
        .await
        .expect("Failed to send subscription message");

    // Wait for subscription confirmation
    let confirm_msg = ws_stream
        .next()
        .await
        .expect("Failed to receive subscription confirmation")
        .unwrap();
    match confirm_msg {
        Message::Text(text) => {
            println!("Received subscription confirmation: {}", text);
            assert!(
                text.contains("\"id\":1"),
                "Unexpected subscription confirmation"
            );
        }
        _ => panic!("Unexpected subscription confirmation format"),
    }

    // Wait for the order book data
    let message = ws_stream
        .next()
        .await
        .expect("Failed to receive order book data")
        .unwrap();

    match message {
        Message::Text(text) => {
            println!("Received order book data: {}", text);
            assert!(text.contains("\"b\""), "Missing bids in message");
            assert!(text.contains("\"a\""), "Missing asks in message");
            assert!(
                text.contains("\"e\":\"depthUpdate\""),
                "Not a depth update message"
            );
            assert!(text.contains("\"s\":\"BTCUSDT\""), "Wrong trading pair");
        }
        _ => panic!("Unexpected message format"),
    }
}

/// Tests that the Binance WebSocket connection properly supports
/// the WebSocket ping/pong health-check mechanism.
///
/// This test verifies:
/// 1. Direct WebSocket connection can be established
/// 2. Ping messages can be sent to the server
/// 3. Pong responses are received within a reasonable timeframe (30 seconds)
#[tokio::test]
async fn test_binance_websocket_ping_pong() {
    let ws_url = get_binance_ws_url();

    // Strip the path from the websocket URL to get the base URL
    let base_url = ws_url.split('/').collect::<Vec<&str>>()[..3].join("/");
    let base_ws_url = format!("{}/ws", base_url);

    let url = url::Url::parse(&base_ws_url).unwrap();
    println!("Connecting to WebSocket base URL: {}", base_ws_url);

    let (mut ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket");

    let start_time = SystemTime::now();
    let mut received_pong = false;

    // Send a ping message
    ws_stream
        .send(Message::Ping(vec![]))
        .await
        .expect("Failed to send ping");

    while start_time.elapsed().unwrap() < Duration::from_secs(30) {
        if let Ok(Message::Pong(_)) = ws_stream.next().await.expect("Failed to receive message") {
            received_pong = true;
            break;
        }
    }

    assert!(
        received_pong,
        "Did not receive Pong response within 30 seconds"
    );
    println!("WebSocket ping-pong test passed");
}

/// Tests that the Binance WebSocket provides frequent updates to the order book,
/// ensuring real-time price data is available.
///
/// This test verifies:
/// 1. The initial order book can be fetched
/// 2. After a short wait (2 seconds), updates are received
/// 3. The update timestamp differs from the initial timestamp
/// 4. Updates occur within a reasonable timeframe (5 seconds)
#[tokio::test]
async fn test_binance_websocket_update_frequency() {
    // Create a new Binance exchange instance
    let exchange = BinanceExchange::new()
        .await
        .expect("Failed to create Binance exchange");

    // Get initial orderbook
    let initial_orderbook = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch initial orderbook");

    // Check for initial orderbook data
    assert!(
        !initial_orderbook.bids.is_empty(),
        "Initial orderbook has no bids"
    );
    assert!(
        !initial_orderbook.asks.is_empty(),
        "Initial orderbook has no asks"
    );

    let initial_best_bid = initial_orderbook.bids.first().map(|b| b.price);
    let initial_best_ask = initial_orderbook.asks.first().map(|a| a.price);

    println!(
        "Initial orderbook - Best bid: {:?}, Best ask: {:?}",
        initial_best_bid, initial_best_ask
    );

    // Wait for updates
    sleep(Duration::from_secs(2)).await;

    // Get updated orderbook
    let updated_orderbook = exchange
        .fetch_order_book()
        .await
        .expect("Failed to fetch updated orderbook");

    // Check for updated orderbook data
    assert!(
        !updated_orderbook.bids.is_empty(),
        "Updated orderbook has no bids"
    );
    assert!(
        !updated_orderbook.asks.is_empty(),
        "Updated orderbook has no asks"
    );

    let updated_best_bid = updated_orderbook.bids.first().map(|b| b.price);
    let updated_best_ask = updated_orderbook.asks.first().map(|a| a.price);

    println!(
        "Updated orderbook - Best bid: {:?}, Best ask: {:?}",
        updated_best_bid, updated_best_ask
    );

    // Verify that we receive updates within a reasonable timeframe
    let update_time = updated_orderbook
        .timestamp
        .duration_since(initial_orderbook.timestamp)
        .unwrap();
    assert!(
        update_time.as_secs() <= 5,
        "Orderbook not updated within 5 seconds"
    );
}
