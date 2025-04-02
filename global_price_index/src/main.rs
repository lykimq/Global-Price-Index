//! Global BTC/USDT Price Index API Binary
//!
//! This is the main entry point for the Global BTC/USDT Price Index API server.

use global_price_index::{config, start_server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize config (happens implicitly via lazy_static)
    // Log configuration values
    println!("Starting Global BTC/USDT Price Index API ...");
    println!("Server address: {}", config::get_server_addr());
    println!("Binance WebSocket URL: {}", config::get_binance_ws_url());

    // Start the server
    start_server().await
}
