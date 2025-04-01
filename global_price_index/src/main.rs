//! Global BTC/USDT Price Index API Binary
//!
//! This is the main entry point for the Global BTC/USDT Price Index API server.

use global_price_index::start_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Global BTC/USDT Price Index API ...");
    start_server().await
}
