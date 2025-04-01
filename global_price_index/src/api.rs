// Exchange trait, factory

use crate::exchanges::{
    binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange, Exchange,
};
use crate::models::GlobalPriceIndex;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub binance: Arc<BinanceExchange>,
    pub kraken: Arc<KrakenExchange>,
    pub huobi: Arc<HuobiExchange>,
}

impl AppState {
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

pub async fn index() -> impl Responder {
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend".to_string());
    let templates_dir = env::var("TEMPLATES_DIR").unwrap_or_else(|_| "templates".to_string());
    let index_html = env::var("INDEX_HTML").unwrap_or_else(|_| "index.html".to_string());

    let path = format!("./{}/{}/{}", frontend_dir, templates_dir, index_html);
    fs::NamedFile::open_async(path).await
}

pub async fn start_server() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Get server configuration from environment
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    // Get frontend paths from environment
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend".to_string());
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
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
