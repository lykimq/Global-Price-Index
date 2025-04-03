use actix_web::{test, web};
use global_price_index::{
    exchanges::{binance::BinanceExchange, huobi::HuobiExchange, kraken::KrakenExchange},
    models::GlobalPriceIndex,
};
use std::sync::Arc;
use std::time::SystemTime;

/// Tests the main global price endpoint to ensure it correctly
/// aggregates price data from all exchanges and returns a valid response.
///
/// This test verifies:
/// 1. The endpoint returns a successful HTTP status
/// 2. The response body contains a valid GlobalPriceIndex JSON
/// 3. The global price is positive and reasonable
/// 4. The timestamp is current
/// 5. Exchange prices are included and valid
#[actix_web::test]
async fn test_global_price_endpoint() {
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

    // Create test app
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(global_price_index::api::AppState {
                binance,
                kraken,
                huobi,
            }))
            .route(
                "/global-price",
                web::get().to(global_price_index::api::get_global_price),
            ),
    )
    .await;

    // Test the endpoint
    let req = test::TestRequest::get().uri("/global-price").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify the response is successful
    assert!(resp.status().is_success());

    // Parse the response body
    let body = test::read_body(resp).await;
    let global_index: GlobalPriceIndex = serde_json::from_slice(&body).unwrap();

    // Verify global price index structure
    assert!(global_index.price > 0.0);
    assert!(global_index.timestamp <= SystemTime::now());
    assert!(!global_index.exchange_prices.is_empty());

    // Verify individual exchange prices are present
    for price in global_index.exchange_prices {
        assert!(price.mid_price > 0.0);
        assert!(price.timestamp <= SystemTime::now());
    }
}

/// Tests that the API properly handles error cases, specifically
/// responding with appropriate error status codes for invalid paths.
///
/// This test verifies:
/// 1. The API returns a client error (4xx) status code for invalid paths
#[actix_web::test]
async fn test_error_handling() {
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

    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(global_price_index::api::AppState {
                binance,
                kraken,
                huobi,
            }))
            .route(
                "/global-price",
                web::get().to(global_price_index::api::get_global_price),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/invalid-path").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
}
