// Exchange trait, factory

use crate::config::{
    get_frontend_dir, get_index_html, get_server_addr, get_static_dir, get_templates_dir,
};
use crate::exchanges::{
    binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange, Exchange,
};
use crate::models::GlobalPriceIndex;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;

/// AppState holds references to all exchange instances
///
/// This struct is shared across HTTP requests and contains
/// thread-safe references to each exchange implementation.
/// It allows the API handlers to access exchange data without
/// creating new exchange instances for each request.
#[derive(Clone)]
pub struct AppState {
    pub binance: Arc<BinanceExchange>,
    pub kraken: Arc<KrakenExchange>,
    pub huobi: Arc<HuobiExchange>,
}

impl AppState {
    /// Creates a new AppState with the provided exchange instances
    ///
    /// Args:
    ///   binance: Arc-wrapped BinanceExchange
    ///   kraken: Arc-wrapped KrakenExchange
    ///   huobi: Arc-wrapped HuobiExchange
    ///
    /// Returns:
    ///   A new AppState instance
    pub fn new(
        binance: Arc<BinanceExchange>,
        kraken: Arc<KrakenExchange>,
        huobi: Arc<HuobiExchange>,
    ) -> Self {
        Self {
            binance,
            kraken,
            huobi,
        }
    }
}

/// HTTP handler for the /global-price endpoint
///
/// This function:
/// 1. Fetches prices from all exchanges
/// 2. Gracefully handles individual exchange failures
/// 3. Creates a GlobalPriceIndex with time-based weighting
/// 4. Returns the index as JSON response
///
/// Returns:
///   HTTP 200 with GlobalPriceIndex JSON on success
///   HTTP 503 if no exchange prices are available
pub async fn get_global_price(data: web::Data<AppState>) -> impl Responder {
    // Create a vector to store the prices from all exchanges
    let mut exchange_prices = Vec::new();

    // Fetch prices from all exchanges
    match data.binance.get_mid_price().await {
        Ok(price) => {
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Binance price: {}", e),
    }

    // Fetch prices from Kraken
    match data.kraken.get_mid_price().await {
        Ok(price) => {
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Kraken price: {}", e),
    }

    // Fetch prices from Huobi
    match data.huobi.get_mid_price().await {
        Ok(price) => {
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Huobi price: {}", e),
    }

    // Check if there is any price data available
    if exchange_prices.is_empty() {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "No price data available from any exchange",
        }));
    }

    // Create the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);

    HttpResponse::Ok().json(global_index)
}

/// HTTP handler for the root (/) endpoint
///
/// Serves the index.html file from the templates directory
///
/// Returns:
///   The index.html file as a response
pub async fn index() -> impl Responder {
    let frontend_dir = get_frontend_dir();
    let templates_dir = get_templates_dir();
    let index_html = get_index_html();

    let path = format!("./{}/{}/{}", frontend_dir, templates_dir, index_html);
    fs::NamedFile::open_async(path).await
}

/// Starts the HTTP server with all routes and exchange instances
///
/// This function:
/// 1. Initializes all exchange connections
/// 2. Sets up API routes and static file serving
/// 3. Starts the server on the configured address
///
/// Returns:
///   std::io::Result<()>: Ok on successful server exit, Err otherwise
pub async fn start_server() -> std::io::Result<()> {
    // Get server address from config
    let addr = get_server_addr();

    // Get frontend paths from config
    let frontend_dir = get_frontend_dir();
    let static_dir = get_static_dir();
    let static_path = format!("./{}/{}", frontend_dir, static_dir);

    // Initialize exchanges
    let binance = Arc::new(
        BinanceExchange::new()
            .await
            .expect("Failed to create Binance exchange"),
    );
    let kraken = Arc::new(
        KrakenExchange::new()
            .await
            .expect("Failed to create Kraken exchange"),
    );
    let huobi = Arc::new(
        HuobiExchange::new()
            .await
            .expect("Failed to create Huobi exchange"),
    );

    // Create the app state
    let app_state = web::Data::new(AppState::new(binance, kraken, huobi));

    // Start the server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/global-price", web::get().to(get_global_price))
            .service(
                fs::Files::new("/static", &static_path)
                    .show_files_listing()
                    .prefer_utf8(true)
                    .use_last_modified(true),
            )
    })
    .bind(&addr)?
    .run()
    .await
}
