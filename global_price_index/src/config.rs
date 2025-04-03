use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::RwLock;
use std::time::Duration;

// Initialize global configuration
lazy_static! {
    /// Global configuration instance that is initialized once and can be accessed from anywhere
    ///
    /// Uses lazy_static for one-time initialization and RwLock for thread-safe access.
    /// The configuration is loaded from config.toml or falls back to default values.
    pub static ref SETTINGS: RwLock<Settings> =
        RwLock::new(Settings::new().expect("Failed to load configuration"));
}

/// Server configuration settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub api_host: String,
    pub api_port: u16,
    pub frontend_host: String,
    pub frontend_port: u16,
}

/// Frontend paths and file locations
#[derive(Debug, Deserialize, Clone)]
pub struct Frontend {
    pub dir: String,
    pub static_dir: String,
    pub templates_dir: String,
    pub index_html: String,
}

/// Binance-specific configuration
#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub ws_url: String,
    pub rest_url: String,
}

/// Kraken-specific configuration
#[derive(Debug, Deserialize, Clone)]
pub struct KrakenConfig {
    pub url: String,
}

/// Huobi-specific configuration
#[derive(Debug, Deserialize, Clone)]
pub struct HuobiConfig {
    pub url: String,
}

/// Common exchange configuration parameters
#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub initial_reconnect_delay: u64,
    pub ping_interval: u64,
    pub max_reconnect_delay: u64,
    pub ping_retry_count: u32,
}

/// Time-based price weighting configuration
#[derive(Debug, Deserialize, Clone)]
pub struct PriceWeighting {
    pub decay_factor: f64,
}

/// Exchange-specific configurations
#[derive(Debug, Deserialize, Clone)]
pub struct Exchange {
    pub binance: BinanceConfig,
    pub kraken: KrakenConfig,
    pub huobi: HuobiConfig,
    pub config: ExchangeConfig,
}

/// Main settings structure that contains all configuration sections
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub frontend: Frontend,
    pub exchange: Exchange,
    pub price_weighting: PriceWeighting,
}

impl Settings {
    /// Creates a new Settings instance by loading from config.toml or using defaults
    ///
    /// This function attempts to load configuration from a file,
    /// and falls back to default values if the file is not found or has errors.
    ///
    /// Returns:
    ///   Result<Self, ConfigError>: The settings or a configuration error
    pub fn new() -> Result<Self, ConfigError> {
        // Try to load config file
        let config_builder = Config::builder().add_source(File::with_name("config"));

        // Attempt to build the configuration from file
        let config_result = config_builder.build();

        match config_result {
            Ok(config) => {
                // Successfully loaded config file, deserialize it
                config.try_deserialize()
            }
            Err(err) => {
                // Config file not found or error loading, use default values
                eprintln!(
                    "Warning: Could not load config file: {}, using default values",
                    err
                );

                Ok(Self {
                    server: Server {
                        api_host: "127.0.0.1".to_string(),
                        api_port: 8080,
                        frontend_host: "127.0.0.1".to_string(),
                        frontend_port: 8081,
                    },
                    frontend: Frontend {
                        dir: "frontend".to_string(),
                        static_dir: "static".to_string(),
                        templates_dir: "templates".to_string(),
                        index_html: "index.html".to_string(),
                    },
                    exchange: Exchange {
                        binance: BinanceConfig {
                            ws_url: "wss://stream.binance.com:9443/ws/btcusdt@depth".to_string(),
                            rest_url:
                                "https://api.binance.com/api/v3/depth?symbol=BTCUSDT&limit=1000"
                                    .to_string(),
                        },
                        kraken: KrakenConfig {
                            url: "https://api.kraken.com/0/public/Depth?pair=XBTUSDT".to_string(),
                        },
                        huobi: HuobiConfig {
                            url: "https://api.huobi.pro/market/depth".to_string(),
                        },
                        config: ExchangeConfig {
                            initial_reconnect_delay: 1,
                            ping_interval: 30,
                            max_reconnect_delay: 300,
                            ping_retry_count: 3,
                        },
                    },
                    price_weighting: PriceWeighting {
                        decay_factor: 300.0, // 5 minutes default
                    },
                })
            }
        }
    }

    /// Reloads configuration from the file
    ///
    /// This function loads the latest configuration from disk
    /// and updates the global SETTINGS instance.
    ///
    /// Returns:
    ///   Result<(), ConfigError>: Success or a configuration error
    pub fn reload() -> Result<(), ConfigError> {
        let settings = Settings::new()?;
        let mut write_guard = SETTINGS.write().unwrap();
        *write_guard = settings;
        Ok(())
    }
}

// Convenience methods to get configuration values

/// Returns the Binance WebSocket URL
pub fn get_binance_ws_url() -> String {
    SETTINGS.read().unwrap().exchange.binance.ws_url.clone()
}

/// Returns the Binance REST API URL
pub fn get_binance_rest_url() -> String {
    SETTINGS.read().unwrap().exchange.binance.rest_url.clone()
}

/// Returns the Kraken API URL
pub fn get_kraken_url() -> String {
    SETTINGS.read().unwrap().exchange.kraken.url.clone()
}

/// Returns the Huobi API URL
pub fn get_huobi_url() -> String {
    SETTINGS.read().unwrap().exchange.huobi.url.clone()
}

/// Returns the initial reconnect delay as a Duration
pub fn get_initial_reconnect_delay() -> Duration {
    Duration::from_secs(
        SETTINGS
            .read()
            .unwrap()
            .exchange
            .config
            .initial_reconnect_delay,
    )
}

/// Returns the WebSocket ping interval as a Duration
pub fn get_ping_interval() -> Duration {
    Duration::from_secs(SETTINGS.read().unwrap().exchange.config.ping_interval)
}

/// Returns the maximum reconnect delay as a Duration
pub fn get_max_reconnect_delay() -> Duration {
    Duration::from_secs(SETTINGS.read().unwrap().exchange.config.max_reconnect_delay)
}

/// Returns the ping retry count
pub fn get_ping_retry_count() -> u32 {
    SETTINGS.read().unwrap().exchange.config.ping_retry_count
}

/// Returns the decay factor for time-based price weighting
pub fn get_decay_factor() -> f64 {
    SETTINGS.read().unwrap().price_weighting.decay_factor
}

/// Returns the API server address in format "host:port"
pub fn get_api_server_addr() -> String {
    let settings = SETTINGS.read().unwrap();
    format!("{}:{}", settings.server.api_host, settings.server.api_port)
}

/// Returns the frontend server address in format "host:port"
pub fn get_frontend_server_addr() -> String {
    let settings = SETTINGS.read().unwrap();
    format!(
        "{}:{}",
        settings.server.frontend_host, settings.server.frontend_port
    )
}

/// Returns the API server URL for CORS configuration
pub fn get_api_server_url() -> String {
    let settings = SETTINGS.read().unwrap();
    format!(
        "http://{}:{}",
        settings.server.api_host, settings.server.api_port
    )
}

/// Returns the frontend server URL for CORS configuration
pub fn get_frontend_server_url() -> String {
    let settings = SETTINGS.read().unwrap();
    format!(
        "http://{}:{}",
        settings.server.frontend_host, settings.server.frontend_port
    )
}

/// Returns the frontend directory path
pub fn get_frontend_dir() -> String {
    SETTINGS.read().unwrap().frontend.dir.clone()
}

/// Returns the static files directory path
pub fn get_static_dir() -> String {
    SETTINGS.read().unwrap().frontend.static_dir.clone()
}

/// Returns the templates directory path
pub fn get_templates_dir() -> String {
    SETTINGS.read().unwrap().frontend.templates_dir.clone()
}

/// Returns the index HTML file name
pub fn get_index_html() -> String {
    SETTINGS.read().unwrap().frontend.index_html.clone()
}
