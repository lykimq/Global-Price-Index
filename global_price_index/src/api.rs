// Exchange trait, factory

use crate::exchanges::{
    binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange, Exchange,
};
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files as fs;
use std::sync::Arc;
use crate::models::GlobalPriceIndex;

pub struct AppState {
    binance: Arc<BinanceExchange>,
    kraken: Arc<KrakenExchange>,
    huobi: Arc<HuobiExchange>,
}

async fn get_global_price(data: web::Data<AppState>) -> impl Responder {
    // Create a vector to store the prices from all exchanges
    let mut exchange_prices = Vec::new();

    // Fetch prices from all exchanges
    if let Ok(price) = data.binance.get_mid_price().await {
        exchange_prices.push(price);
    }

    // Fetch prices from Kraken
    if let Ok(price) = data.kraken.get_mid_price().await {
        exchange_prices.push(price);
    }

    // Fetch prices from Huobi
    if let Ok(price) = data.huobi.get_mid_price().await {
        exchange_prices.push(price);
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

async fn index() -> impl Responder {
    fs::NamedFile::open_async("./templates/index.html").await
}

pub async fn start_server() -> std::io::Result<()> {
    // Initialize exchanges
    let binance = Arc::new(BinanceExchange::new().await.expect("Failed to create Binance exchange"));
    let kraken = Arc::new(KrakenExchange::new().await.expect("Failed to create Kraken exchange"));
    let huobi = Arc::new(HuobiExchange::new().await.expect("Failed to create Huobi exchange"));

    // Create the app state
    let app_state = web::Data::new(AppState {
        binance,
        kraken,
        huobi,
    });

    // Start the server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/global-price", web::get().to(get_global_price))
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

