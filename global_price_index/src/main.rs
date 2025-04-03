//! Global BTC/USDT Price Index API Binary
//!
//! This is the main entry point for the Global BTC/USDT Price Index API server.

use actix_files as fs;
use actix_web::{middleware, App, HttpServer};
use futures::future::try_join;
use global_price_index::{config, api::start_server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize config (happens implicitly via lazy_static)
    // Log configuration values
    println!("Starting Global BTC/USDT Price Index API ...");
    println!("API server address: {}", config::get_api_server_addr());
    println!("Frontend server address: {}", config::get_frontend_server_addr());
    println!("Binance WebSocket URL: {}", config::get_binance_ws_url());

    // Get frontend paths from config
    let frontend_dir = config::get_frontend_dir();
    let templates_dir = config::get_templates_dir();
    let static_dir = config::get_static_dir();

    // Set up paths for serving
    let templates_path = format!("./{}/{}", frontend_dir, templates_dir);
    let static_path = format!("./{}/{}", frontend_dir, static_dir);

    // Start the API server
    let api_server = start_server().await?;

    // Start the static file server
    println!("Starting static file server...");
    let static_server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            // Serve static assets from static directory
            .service(
                fs::Files::new("/static", &static_path)
                    .show_files_listing()
                    .use_last_modified(true)
            )
            // Serve index.html from templates directory
            .service(
                fs::Files::new("/", &templates_path)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .use_last_modified(true)
            )
    })
    .bind(config::get_frontend_server_addr())?
    .run();

    // Run both servers
    try_join(api_server, static_server).await?;

    Ok(())
}
