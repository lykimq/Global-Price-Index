// WebSocket client, order book sync
use crate::exchanges::Exchange;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{PriceIndexError, Result};
use crate::models::OrderBook;
use std::time::{SystemTime, Duration};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use tokio::time::sleep;

const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws/btcusdt@depth";
const BINANCE_REST_URL: &str = "https://api.binance.com/api/v3/depth?symbol=BTCUSDT&limit=1000";
const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const PING_INTERVAL: Duration = Duration::from_secs(30); // Send ping every 30 seconds

#[derive(Debug, Serialize, Deserialize)]
struct BinanceOrderBook {
    #[serde(rename = "bids")]
    bids: Vec<[String; 2]>, // [price, quantity]

    #[serde(rename = "asks")]
    asks: Vec<[String; 2]>, // [price, quantity]

    #[serde(rename = "lastUpdateId")]
    last_update_id: i64, // Last update ID
}

#[derive(Clone)]
pub struct BinanceExchange {
    order_book: Arc<RwLock<OrderBook>>,
}

impl BinanceExchange {
    // Create a new Binance exchange instance
    pub async fn new() -> Result<Self> {
        let order_book = Arc::new(RwLock::new(OrderBook {
            bids: vec![],
            asks: vec![],
            timestamp: SystemTime::now(),
        }));
        let exchange = Self {order_book};

        exchange.initialize().await?;
        Ok (exchange)
    }

    // Initialize the exchange by fetching initial order book data
    async fn initialize(&self) -> Result<()> {
        // Fetch initial order book data from Binance REST API
        let client = reqwest::Client::new();
        let response : BinanceOrderBook =
        client
            .get(BINANCE_REST_URL)
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

    async fn connect_websocket() -> Result<(futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>, futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>)> {
        let url = Url::parse(BINANCE_WS_URL).map_err(|e| {
            PriceIndexError::WebSocketError(format!("Failed to parse WebSocket URL: {}", e))
        })?;

        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            PriceIndexError::WebSocketError(format!("Failed to connect to WebSocket: {}", e))
        })?;

        Ok(ws_stream.split())
    }

    async fn handle_websocket_messages(
        mut read: futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
        mut write: futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
        order_book: Arc<RwLock<OrderBook>>,
    ) {
        let mut last_pong = SystemTime::now();
        let mut ping_interval = tokio::time::interval(PING_INTERVAL);

        loop {
            tokio::select! {
                Some(message) = read.next() => {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(update) = serde_json::from_str::<BinanceOrderBook>(&text) {
                                let mut order_book = order_book.write().await;
                                // Only update if we have valid data
                                if !update.bids.is_empty() && !update.asks.is_empty() {
                                    order_book.bids = update.bids;
                                    order_book.asks = update.asks;
                                    order_book.timestamp = SystemTime::now();
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            eprintln!("WebSocket connection closed");
                            break;
                        }
                        Ok(Message::Ping(payload)) => {
                            // Respond to ping with pong
                            if let Err(e) = write.send(Message::Pong(payload)).await {
                                eprintln!("Failed to send pong response: {}", e);
                                break;
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            last_pong = SystemTime::now();
                            eprintln!("Received pong, connection is healthy");
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
                    if last_pong.elapsed().unwrap_or(Duration::from_secs(0)) > PING_INTERVAL * 2 {
                        eprintln!("No pong received for too long, reconnecting...");
                        break;
                    }
                    // Send a ping to keep the connection alive
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        eprintln!("Failed to send ping: {}", e);
                        break;
                    }
                }
            }
        }
    }

    // Start the WebSocket connection to receive real-time updates
    async fn start_websocket(&self) -> Result<()> {
        let order_book = self.order_book.clone();
        let mut reconnect_attempt = 0;
        let mut reconnect_delay = INITIAL_RECONNECT_DELAY;

        tokio::spawn(async move {
            loop {
                match Self::connect_websocket().await {
                    Ok((write, read)) => {
                        // Reset reconnection parameters on successful connection
                        reconnect_attempt = 0;
                        reconnect_delay = INITIAL_RECONNECT_DELAY;
                        Self::handle_websocket_messages(read, write, order_book.clone()).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to WebSocket: {}", e);
                    }
                }

                // Implement exponential backoff for reconnection
                if reconnect_attempt < MAX_RECONNECT_ATTEMPTS {
                    eprintln!(
                        "Attempting to reconnect in {} seconds (attempt {}/{})",
                        reconnect_delay.as_secs(),
                        reconnect_attempt + 1,
                        MAX_RECONNECT_ATTEMPTS
                    );
                    sleep(reconnect_delay).await;
                    reconnect_attempt += 1;
                    reconnect_delay *= 2; // Exponential backoff
                } else {
                    eprintln!("Max reconnection attempts reached. Stopping reconnection attempts.");
                    break;
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl Exchange for BinanceExchange {
    // Get the name of the exchange
    fn name(&self) -> &'static str {
        "Binance"
    }

    // Fetch the current order book
    async fn fetch_order_book (&self) -> Result<OrderBook>{
        Ok(self.order_book.read().await.clone())
    }
}
