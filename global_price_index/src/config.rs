use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::RwLock;
use std::time::Duration;

// Initialize global configuration
lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> =
        RwLock::new(Settings::new().expect("Failed to load configuration"));
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Frontend {
    pub dir: String,
    pub static_dir: String,
    pub templates_dir: String,
    pub index_html: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub ws_url: String,
    pub rest_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KrakenConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HuobiConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub initial_reconnect_delay: u64,
    pub ping_interval: u64,
    pub max_reconnect_delay: u64,
    pub ping_retry_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Exchange {
    pub binance: BinanceConfig,
    pub kraken: KrakenConfig,
    pub huobi: HuobiConfig,
    pub config: ExchangeConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub frontend: Frontend,
    pub exchange: Exchange,
}

impl Settings {
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
                        host: "127.0.0.1".to_string(),
                        port: 8080,
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
                })
            }
        }
    }

    // Helper method to reload configuration
    pub fn reload() -> Result<(), ConfigError> {
        let settings = Settings::new()?;
        let mut write_guard = SETTINGS.write().unwrap();
        *write_guard = settings;
        Ok(())
    }
}

// Convenience methods to get configuration values
pub fn get_binance_ws_url() -> String {
    SETTINGS.read().unwrap().exchange.binance.ws_url.clone()
}

pub fn get_binance_rest_url() -> String {
    SETTINGS.read().unwrap().exchange.binance.rest_url.clone()
}

pub fn get_kraken_url() -> String {
    SETTINGS.read().unwrap().exchange.kraken.url.clone()
}

pub fn get_huobi_url() -> String {
    SETTINGS.read().unwrap().exchange.huobi.url.clone()
}

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

pub fn get_ping_interval() -> Duration {
    Duration::from_secs(SETTINGS.read().unwrap().exchange.config.ping_interval)
}

pub fn get_max_reconnect_delay() -> Duration {
    Duration::from_secs(SETTINGS.read().unwrap().exchange.config.max_reconnect_delay)
}

pub fn get_ping_retry_count() -> u32 {
    SETTINGS.read().unwrap().exchange.config.ping_retry_count
}

pub fn get_server_addr() -> String {
    let settings = SETTINGS.read().unwrap();
    format!("{}:{}", settings.server.host, settings.server.port)
}

pub fn get_frontend_dir() -> String {
    SETTINGS.read().unwrap().frontend.dir.clone()
}

pub fn get_static_dir() -> String {
    SETTINGS.read().unwrap().frontend.static_dir.clone()
}

pub fn get_templates_dir() -> String {
    SETTINGS.read().unwrap().frontend.templates_dir.clone()
}

pub fn get_index_html() -> String {
    SETTINGS.read().unwrap().frontend.index_html.clone()
}
