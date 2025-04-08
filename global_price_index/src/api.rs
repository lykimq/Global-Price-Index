// Exchange trait, factory

use crate::config::{get_api_server_addr, get_frontend_server_url};
use crate::exchanges::{
    binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange, Exchange,
};
use crate::models::GlobalPriceIndex;
use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, HttpResponse, HttpServer, Responder};
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

/// Configures the API routes and state
///
/// This function:
/// 1. Initializes exchange connections
/// 2. Sets up the AppState
/// 3. Configures the /global-price route
pub async fn configure_api_routes() -> AppState {
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

    // Create and return the app state
    AppState::new(binance, kraken, huobi)
}

/// Starts the HTTP server with API routes and exchange instances
///
/// This function:
/// 1. Initializes all exchange connections
/// 2. Sets up the /global-price API route with CORS support
/// 3. Starts the server
pub async fn start_server() -> std::io::Result<actix_web::dev::Server> {
    // Get server address from config
    let addr = get_api_server_addr();
    let frontend_url = get_frontend_server_url();

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

    // Create and start the server
    Ok(HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_origin(&frontend_url.replace("127.0.0.1", "localhost"))
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .route("/global-price", web::get().to(get_global_price))
    })
    .bind(&addr)?
    .run())
}
