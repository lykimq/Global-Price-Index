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
    println!("Fetching Binance price...");
    match data.binance.get_mid_price().await {
        Ok(price) => {
            println!("Binance price: ${:.2}", price.mid_price);
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Binance price: {}", e),
    }

    // Fetch prices from Kraken
    println!("Fetching Kraken price...");
    match data.kraken.get_mid_price().await {
        Ok(price) => {
            println!("Kraken price: ${:.2}", price.mid_price);
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Kraken price: {}", e),
    }

    // Fetch prices from Huobi
    println!("Fetching Huobi price...");
    match data.huobi.get_mid_price().await {
        Ok(price) => {
            println!("Huobi price: ${:.2}", price.mid_price);
            exchange_prices.push(price);
        }
        Err(e) => println!("Error fetching Huobi price: {}", e),
    }

    // Check if there is any price data available
    if exchange_prices.is_empty() {
        println!("No exchange prices available!");
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "No price data available from any exchange",
        }));
    }

    // Create the global price index
    let global_index = GlobalPriceIndex::new(exchange_prices);
    println!("Global price index: ${:.2}", global_index.price);
    HttpResponse::Ok().json(global_index)
}

async fn index() -> impl Responder {
    fs::NamedFile::open_async("./frontend/templates/index.html").await
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
        println!("Setting up static file serving from ./frontend/static");
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/global-price", web::get().to(get_global_price))
            .service(
                fs::Files::new("/static", "./frontend/static")
                    .show_files_listing()
                    .prefer_utf8(true)
                    .use_last_modified(true)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

