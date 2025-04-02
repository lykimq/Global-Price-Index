# BTC/USDT Global Price Index

A high-performance, real-time global price index service for BTC/USDT that aggregates orderbook data from multiple cryptocurrency exchanges. Built with Rust and Actix Web.

## Overview

This service computes a global BTC/USDT price index by:
1. Fetching order books from:
- Binance: Real-time WebSocket stream (`btcusdt@depth`) with snapshot initialization and incremental updates.
- Kraken and Huobi: REST API polling (best bid/ask).
2. Calculating mid-prices for each exchange.
3. Aggregating results into a global index.

**Key Requirements Addressed**:
- WebSocket integration for Binance (snapshot + updates).
- REST APIs for Kraken/Huobi.
- Thread-safe state management (`Arc<RwLock<OrderBook>>`).
- Extensible architecture for new exchanges.

## Features

- Real-time Data:
    + Binance: WebSocket stream with snapshot recovery.
    + Kraken/Huobi: REST polling (configurable interval).
- Mid-Price Calculation:
```
mid_price = (best_bid + best_ask)/2
```
Validation: Skips invalid data (empty bids/asks).
- Global Index:
    + Average of valid mid-prices (ignores failed exchanges).
- Fault Tolerance:
    + Retries for failed API calls (exponential backoff).
    + WebSocket reconnection with exponential backoff.
- Testing:
    + Unit tests: Test order book parsing, mid-price calculation, and data validation.
    + WebSocket tests: Test WebSocket connection, reconnection, message format, and ping/pong mechanisms.
    + Integration tests: Test API endpoints and end-to-end functionality.
    + Property tests: Test data model properties and invariants using proptest framework.

## Frontend Access

The web interface for the Global Price Index is accessible at:
```
http://localhost:8080
```

**Note on HTTP vs HTTPS:**
- The service currently uses HTTP for its local web interface
- This is appropriate for development and internal network usage
- For production deployment, configure with TLS/HTTPS using a reverse proxy like Nginx or an application load balancer

## API Endpoints

**Global Price Index**

```
GET /global-price
```

**Response**
```
{
    "price": 84640.55333333333,
    "timestamp":1743583727.3289757,
    "exchange_prices":
    [
        {
            "exchange": "Binance",
            "mid_price":84642.0,
            "timestamp":1743583726.9247916},
        {
            "exchange":"Kraken",
            "mid_price":84648.15,
            "timestamp":1743583726.9877129
        },
        {
            "exchange":"Huobi",
            "mid_price":84631.51,
            "timestamp":1743583727.3289661
        }
    ]
}
```
Notes:
- **Fault Tolerance:** The service gracefully handles exchange failures:
  + If any single exchange fails (e.g., Huobi, Binance, or Kraken), the global price index uses the average of data from the remaining functioning exchanges.
  + If two exchanges fail, the global price index uses data from the single remaining exchange.
  + If all exchanges fail, the API returns a 503 Service Unavailable response with an error message.

## Security

The current implementation includes several security features:

- **Secure Communication**:
  + Uses HTTPS for outbound REST API calls to exchanges (external communication)
  + Uses WSS (WebSocket Secure) for real-time data streams from exchanges
  + Default TLS verification enabled in HTTP and WebSocket clients for all external APIs
  + Note: Internal web server uses HTTP by default and should be placed behind a TLS-terminating proxy for production

- **TLS Verification Explained**:
  + For HTTP clients: The `reqwest` client verifies TLS certificates by default, ensuring connections to exchanges are secure and properly authenticated
  + For WebSocket clients: The `tokio_tungstenite` library with the `native-tls` feature validates server certificates during WebSocket connection establishment
  + This prevents man-in-the-middle attacks when communicating with exchange APIs

- **Input Validation**:
  + Validates all price data before processing (non-empty, positive values)
  + Verifies bid/ask spreads are reasonable (ask > bid)
  + Enforces proper data formats from exchange APIs

- **Error Handling**:
  + Comprehensive error types for different failure scenarios
  + Avoids exposing internal errors to API consumers
  + Graceful recovery from temporary failures

- **Connection Security**:
  + Configurable timeouts for HTTP requests (5 seconds default)
  + Ping/pong mechanisms to verify WebSocket connection health
  + Automatic reconnection with backoff to prevent overwhelming servers

- **Data Integrity**:
  + Thread-safe access to shared state using `Arc<RwLock<>>`
  + Atomicity in price calculations
  + Timestamp validation and consistent formatting

## Prerequisites

- Rust (latest stable version)
- `make` (for build automation)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd global_price_index
```

2. Install dependencies:
```bash
cargo build --release
# or
make install
```

3. Configure environment variables (`.env`):
```bash
cp .env.example .env
# Edit .env with your configuration
```

## Running

Run the application:
```bash
cargo run --release
# or
make run
```

## Testing

Run all tests:
```bash
cargo test
# or
make test
```

For WebSocket tests (which may take longer):
```bash
cargo test websocket
```

For property-based tests:
```bash
# Run property tests with regression file generation
cargo test --test property_tests -- --test-threads=1

# Run any previously failing tests marked as ignored
cargo test --test property_tests -- --test-threads=1 --ignored
```

## Exchange API References
- Binance API: [Binance WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- Kraken API: [Kraken REST API](https://docs.kraken.com/api/)
- Huobi API: [Huobi REST API](https://www.htx.com/en-us/opend/newApiPages)

**Exchange Integration**

| Exchange | Protocol    | Endpoint                      | Thread Safety                                      |
|----------|-------------|-------------------------------|----------------------------------------------------|
| Binance  | WebSocket   | `btcusdt@depth`               | `Arc<RwLock<OrderBook>>` for persistent state      |
| Kraken   | REST        | `/Depth?pair=XBTUSDT`         | Stateless - thread-safe via `Arc<KrakenExchange>`  |
| Huobi    | REST        | `/market/depth?symbol=btcusdt`| Stateless - thread-safe via `Arc<HuobiExchange>`   |
