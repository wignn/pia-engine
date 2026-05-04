use std::error::Error;
use std::future::Future;

use chrono::Utc;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use tradingview::live::{
    handler::message::TradingViewResponse,
    models::DataServer,
    websocket::WebSocketClient,
};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct DataCollectorConfig {
    pub auth_token: String,
    pub server: DataServer,
    pub symbols: Vec<String>,
}

impl DataCollectorConfig {
    pub fn from_app_config(cfg: &Config) -> Self {
        Self {
            auth_token: cfg.tv_auth_token.clone(),
            server: parse_data_server(&cfg.tv_server),
            symbols: cfg.tv_symbols.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.auth_token.trim().is_empty() {
            return Err("TV_AUTH_TOKEN is empty".to_string());
        }
        if self.symbols.is_empty() {
            return Err("TV_SYMBOLS is empty".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceTick {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub volume: Option<f64>,
    pub timestamp: Option<i64>,
    pub received_at: String,
}

impl PriceTick {
    fn from_quote(default_symbol: &str, quote: &tradingview::QuoteValue) -> Option<Self> {
        Self::from_quote_with_map(default_symbol, &std::collections::HashMap::new(), quote)
    }

    fn from_quote_with_map(
        default_symbol: &str,
        symbol_map: &std::collections::HashMap<String, String>,
        quote: &tradingview::QuoteValue,
    ) -> Option<Self> {
        let price = quote.price?;
    
    let short_name = quote
            .symbol
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let symbol = match short_name {
            Some(ref s) if s.contains(':') => s.clone(),
            Some(ref s) if symbol_map.contains_key(s.as_str()) => {
                symbol_map[s.as_str()].clone()
            }
            Some(ref s) if default_symbol.ends_with(s.as_str()) => {
                default_symbol.to_string()
            }
            Some(s) => s,
            None => default_symbol.to_string(),
        };

        Some(Self {
            symbol,
            price,
            bid: quote.bid,
            ask: quote.ask,
            change: quote.change,
            change_percent: quote.change_percent,
            volume: quote.volume,
            timestamp: quote.timestamp.map(|t| t as i64),
            received_at: Utc::now().to_rfc3339(),
        })
    }
}

pub struct DataCollector {
    config: DataCollectorConfig,
}

impl DataCollector {
    pub fn new(config: DataCollectorConfig) -> Self {
        Self { config }
    }

    pub async fn stream_forever<F, Fut>(
        &self,
        mut on_tick: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: FnMut(PriceTick) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        self.config
            .validate()
            .map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?;

        let default_symbol = self
            .config
            .symbols
            .first()
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // Build a lookup: short_name (after ':') → full symbol
        // e.g. "XAUUSD" -> "OANDA:XAUUSD", "BTCUSDT" -> "BINANCE:BTCUSDT"
        let symbol_map: std::collections::HashMap<String, String> = self
            .config
            .symbols
            .iter()
            .map(|full| {
                let short = full.split(':').last().unwrap_or(full.as_str()).to_string();
                (short, full.clone())
            })
            .collect();

        info!(
            server = %format_data_server(self.config.server),
            symbols = ?self.config.symbols,
            "price stream connecting"
        );

        let (data_tx, mut data_rx) = mpsc::unbounded_channel::<TradingViewResponse>();

        let ws_client = WebSocketClient::builder()
            .auth_token(&self.config.auth_token)
            .server(self.config.server)
            .data_tx(data_tx)
            .build()
            .await?;

        ws_client.clone().spawn_reader_task();
        ws_client.create_quote_session().await?;
        ws_client.set_fields().await?;

        let symbols: Vec<&str> = self.config.symbols.iter().map(String::as_str).collect();
        ws_client.add_symbols(&symbols).await?;

        let mut quote_count: u64 = 0;
        while let Some(msg) = data_rx.recv().await {
            match msg {
                TradingViewResponse::QuoteData(quote) => {
                    if let Some(tick) = PriceTick::from_quote_with_map(&default_symbol, &symbol_map, &quote) {
                        quote_count += 1;
                        on_tick(tick).await;
                    }
                }
                TradingViewResponse::Error(e, context) => {
                    if is_benign_tv_protocol_error(&e.to_string(), &context) {
                        debug!(error = %e, ctx = ?context, "price stream benign protocol event");
                    } else {
                        warn!(error = %e, ctx = ?context, "price stream received error event");
                    }
                }
                _ => {}
            }
        }

        debug!(quote_count, "price stream channel closed, shutting down websocket");

        if let Err(e) = ws_client.delete_quote_session().await {
            error!(error = %e, "failed to delete quote session");
        }
        if let Err(e) = ws_client.close().await {
            error!(error = %e, "failed to close websocket client");
        }

        Ok(())
    }
}

fn parse_data_server(value: &str) -> DataServer {
    match value.trim().to_ascii_lowercase().as_str() {
        "prodata" => DataServer::ProData,
        "widgetdata" => DataServer::WidgetData,
        "mobile-data" | "mobiledata" => DataServer::MobileData,
        _ => DataServer::Data,
    }
}

fn format_data_server(server: DataServer) -> &'static str {
    match server {
        DataServer::Data => "data",
        DataServer::ProData => "prodata",
        DataServer::WidgetData => "widgetdata",
        DataServer::MobileData => "mobile-data",
    }
}

fn is_benign_tv_protocol_error(error: &str, context: &[JsonValue]) -> bool {
    let e = error.to_ascii_lowercase();
    if !e.contains("invalid_method") {
        return false;
    }

    context
        .iter()
        .any(|entry| matches!(entry, JsonValue::String(s) if s.eq_ignore_ascii_case("qsd")))
}
