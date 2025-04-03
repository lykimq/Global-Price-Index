// WebSocket client, order book sync
use crate::config::{
    get_binance_rest_url, get_binance_ws_url, get_initial_reconnect_delay, get_max_reconnect_delay,
    get_ping_interval, get_ping_retry_count,
};
use crate::error::{PriceIndexError, Result};
use crate::exchanges::Exchange;
use crate::models::{Order, OrderBook};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

// Type aliases for WebSocket types
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = futures::stream::SplitSink<WsStream, Message>;
type WsStreamRead = futures::stream::SplitStream<WsStream>;

/// Binance order book structure that matches the Binance API response format
#[derive(Debug, Serialize, Deserialize)]
struct BinanceOrderBook {
    #[serde(rename = "bids", deserialize_with = "deserialize_binance_orders")]
    bids: Vec<Order>,
    #[serde(rename = "asks", deserialize_with = "deserialize_binance_orders")]
    asks: Vec<Order>,

    #[serde(rename = "lastUpdateId")]
    last_update_id: i64, // Last update ID
}

/// Custom deserializer for Binance order book data
///
/// Binance returns orders as [price: String, quantity: String]
/// This function converts them to our Order struct with f64 values
fn deserialize_binance_orders<'de, D>(deserializer: D) -> std::result::Result<Vec<Order>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let raw: Vec<[String; 2]> = Vec::deserialize(deserializer)?;

    raw.into_iter()
        .map(|[price, quantity]| {
            let price = price.parse::<f64>().map_err(|_| {
                D::Error::custom(format!("Failed to parse price as f64: {}", price))
            })?;
            let quantity = quantity.parse::<f64>().map_err(|_| {
                D::Error::custom(format!("Failed to parse quantity as f64: {}", quantity))
            })?;

            Ok(Order { price, quantity })
        })
        .collect()
}

/// The BinanceExchange implements the Exchange trait for Binance
///
/// It uses WebSockets for real-time order book updates and maintains
/// an in-memory order book that is updated incrementally.
#[derive(Clone)]
pub struct BinanceExchange {
    order_book: Arc<RwLock<OrderBook>>,
}

impl BinanceExchange {
    /// Creates a new BinanceExchange instance
    ///
    /// This function:
    /// 1. Creates an empty order book
    /// 2. Initializes the exchange by fetching the initial order book snapshot
    /// 3. Starts a WebSocket connection for real-time updates
    ///
    /// Returns:
    ///   Result<Self>: The exchange instance or an error
    pub async fn new() -> Result<Self> {
        let order_book = Arc::new(RwLock::new(OrderBook {
            bids: vec![],
            asks: vec![],
            timestamp: SystemTime::now(),
        }));
        let exchange = Self { order_book };

        exchange.initialize().await?;
        Ok(exchange)
    }

    /// Initializes the exchange by fetching the initial order book data
    ///
    /// This function:
    /// 1. Fetches the initial order book snapshot from Binance REST API
    /// 2. Updates the in-memory order book with the snapshot data
    /// 3. Starts a WebSocket connection for real-time updates
    async fn initialize(&self) -> Result<()> {
        // Fetch initial order book data from Binance REST API
        let client = reqwest::Client::new();
        let response: BinanceOrderBook = client
            .get(&get_binance_rest_url())
            .send()
            .await?
            .json()
            .await?;

        // Update the order book with the initial data
        let mut order_book = self.order_book.write().await;
        order_book.bids = response.bids;
        order_book.asks = response.asks;
        order_book.timestamp = SystemTime::now();

        // Start WebSocket connection
        self.start_websocket().await?;
        Ok(())
    }

    /// Establishes a WebSocket connection to Binance
    ///
    /// Returns:
    ///   Result<(WsSink, WsStreamRead)>: The WebSocket write and read streams
    async fn connect_websocket() -> Result<(WsSink, WsStreamRead)> {
        let url = Url::parse(&get_binance_ws_url()).map_err(|e| {
            PriceIndexError::WebSocketError(format!("Failed to parse WebSocket URL: {}", e))
        })?;

        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            PriceIndexError::WebSocketError(format!("Failed to connect to WebSocket: {}", e))
        })?;

        Ok(ws_stream.split())
    }

    /// Merges order book updates into the existing order book
    ///
    /// This function:
    /// 1. Applies incremental updates to the order book
    /// 2. Adds new price levels, updates existing ones, or removes levels with zero quantity
    /// 3. Sorts the orders appropriately (bids descending, asks ascending)
    ///
    /// Args:
    ///   existing_orders: The current list of orders to update
    ///   updates: The new orders to apply as updates
    ///   is_bids: Whether we're updating bids (true) or asks (false)
    fn merge_order_book_updates(
        existing_orders: &mut Vec<Order>,
        updates: &[Order],
        is_bids: bool,
    ) {
        // Create a combined list of all orders
        let mut all_orders = existing_orders.clone();

        // Apply updates
        for update in updates {
            let price = update.price;
            let quantity = update.quantity;

            // Check if this price level already exists
            if let Some(existing_idx) = all_orders
                .iter()
                .position(|order| (order.price - price).abs() < f64::EPSILON)
            {
                if quantity > 0.0 {
                    // Update existing order
                    all_orders[existing_idx].quantity = quantity;
                } else {
                    // Remove the order (zero quantity indicates deletion)
                    all_orders.remove(existing_idx);
                }
            } else if quantity > 0.0 {
                // Add new order
                all_orders.push(Order { price, quantity });
            }
        }

        // Sort all orders
        if is_bids {
            // Sort bids in descending order (highest bid first)
            all_orders.sort_by(|a, b| {
                b.price
                    .partial_cmp(&a.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            // Sort asks in ascending order (lowest ask first)
            all_orders.sort_by(|a, b| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Replace the existing orders with the updated ones
        *existing_orders = all_orders;
    }

    /// Handles WebSocket messages and updates the order book
    ///
    /// This function:
    /// 1. Processes incoming WebSocket messages
    /// 2. Updates the order book with incremental changes
    /// 3. Maintains the WebSocket connection with ping/pong messages
    /// 4. Handles connection errors and closures
    async fn handle_websocket_messages(
        mut read: WsStreamRead,
        mut write: WsSink,
        order_book: Arc<RwLock<OrderBook>>,
    ) {
        let mut last_pong = SystemTime::now();
        let mut ping_interval = tokio::time::interval(get_ping_interval());

        println!("WebSocket message handler started");
        loop {
            tokio::select! {
                Some(message) = read.next() => {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(update) = serde_json::from_str::<BinanceOrderBook>(&text) {
                                let mut order_book = order_book.write().await;
                                // Only update if we have valid data
                                if !update.bids.is_empty() && !update.asks.is_empty() {
                                    // Get the current best bid and ask prices if available
                                    let current_best_bid = order_book.bids.first().map(|b| b.price);
                                    let current_best_ask = order_book.asks.first().map(|a| a.price);

                                    // Merge updates rather than replacing entire book
                                    Self::merge_order_book_updates(&mut order_book.bids, &update.bids, true);
                                    Self::merge_order_book_updates(&mut order_book.asks, &update.asks, false);

                                    // Get the new best bid and ask prices
                                    let new_best_bid = order_book.bids.first().map(|b| b.price);
                                    let new_best_ask = order_book.asks.first().map(|a| a.price);

                                    // Log if best prices have changed
                                    if current_best_bid != new_best_bid || current_best_ask != new_best_ask {
                                        println!("Order book top levels updated - Old: {:?}/{:?} New: {:?}/{:?}",
                                            current_best_bid, current_best_ask, new_best_bid, new_best_ask);
                                    }

                                    // Always update the timestamp when we receive valid data
                                    order_book.timestamp = SystemTime::now();
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            eprintln!("WebSocket connection closed");
                            break;
                        }
                        Ok(Message::Ping(payload)) => {
                            // Respond to ping with pong, with retry logic
                            let mut retry_count = 0;
                            let max_retries = get_ping_retry_count();
                            while retry_count < max_retries {
                                match write.send(Message::Pong(payload.clone())).await {
                                    Ok(_) => {
                                        break;
                                    }
                                    Err(e) => {
                                        retry_count += 1;
                                        eprintln!("Failed to send pong response (attempt {}/{}): {}",
                                            retry_count, max_retries, e);
                                        if retry_count >= max_retries {
                                            eprintln!("Max pong retry attempts reached, reconnecting...");
                                            break;
                                        }
                                        sleep(Duration::from_millis(100)).await;
                                    }
                                }
                            }
                            if retry_count >= max_retries {
                                break;
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            last_pong = SystemTime::now();
                            println!("Received pong, connection is healthy");
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                _ = ping_interval.tick() => {
                    // Check if we haven't received a pong for too long
                    if last_pong.elapsed().unwrap_or(Duration::from_secs(0)) > get_ping_interval() * 2 {
                        eprintln!("No pong received for too long, reconnecting...");
                        break;
                    }

                    // Send a ping to keep the connection alive, with retry logic
                    let mut retry_count = 0;
                    let max_retries = get_ping_retry_count();
                    while retry_count < max_retries {
                        match write.send(Message::Ping(vec![])).await {
                            Ok(_) => {
                                break;
                            }
                            Err(e) => {
                                retry_count += 1;
                                eprintln!("Failed to send ping (attempt {}/{}): {}",
                                    retry_count, max_retries, e);
                                if retry_count >= max_retries {
                                    eprintln!("Max ping retry attempts reached, reconnecting...");
                                    break;
                                }
                                sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                    if retry_count >= max_retries {
                        break;
                    }
                }
            }
        }
    }

    /// Starts the WebSocket connection with automatic reconnection
    ///
    /// This function:
    /// 1. Establishes a WebSocket connection to Binance
    /// 2. Spawns a task to handle WebSocket messages
    /// 3. Implements exponential backoff for reconnection attempts
    /// 4. Continues reconnecting indefinitely to maintain data flow
    async fn start_websocket(&self) -> Result<()> {
        let order_book = self.order_book.clone();
        let mut reconnect_attempt = 0;
        let mut reconnect_delay = get_initial_reconnect_delay();
        let max_reconnect_delay = get_max_reconnect_delay();

        tokio::spawn(async move {
            loop {
                match Self::connect_websocket().await {
                    Ok((write, read)) => {
                        // Reset reconnection parameters on successful connection
                        reconnect_attempt = 0;
                        reconnect_delay = get_initial_reconnect_delay();
                        Self::handle_websocket_messages(read, write, order_book.clone()).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to WebSocket: {}", e);
                    }
                }

                // Implement exponential backoff for reconnection with a maximum cap
                eprintln!(
                    "Attempting to reconnect in {} seconds (attempt {})",
                    reconnect_delay.as_secs(),
                    reconnect_attempt + 1
                );
                sleep(reconnect_delay).await;
                reconnect_attempt += 1;

                // Double the delay with a cap at max_reconnect_delay
                reconnect_delay = std::cmp::min(reconnect_delay * 2, max_reconnect_delay);
            }
        });

        Ok(())
    }
}

#[async_trait]
impl Exchange for BinanceExchange {
    /// Returns the name of the exchange
    fn name(&self) -> &'static str {
        "Binance"
    }

    /// Fetches the current order book
    ///
    /// This implementation returns the in-memory order book
    /// that's continuously updated via WebSocket
    async fn fetch_order_book(&self) -> Result<OrderBook> {
        Ok(self.order_book.read().await.clone())
    }
}
