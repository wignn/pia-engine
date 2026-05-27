use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone, Debug, Serialize)]
pub struct ExchangeRule {
    pub exchange_code: String,
    pub name: String,
    pub timezone: String,
    pub regular_open: Option<NaiveTime>,
    pub regular_close: Option<NaiveTime>,
    pub working_days: HashSet<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SymbolExchange {
    pub symbol: String,
    pub exchange_code: String,
    pub asset_type: String,
}

#[derive(Clone, Default)]
pub struct CalendarCache {
    pub exchanges: Arc<RwLock<HashMap<String, ExchangeRule>>>,
    pub symbols: Arc<RwLock<HashMap<String, SymbolExchange>>>,
    pub holidays: Arc<RwLock<HashSet<(String, NaiveDate)>>>,
}

impl CalendarCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exchange_for_symbol(&self, symbol: &str) -> Option<SymbolExchange> {
        self.symbols.read().get(&symbol.to_uppercase()).cloned()
    }

    pub fn exchange_rule(&self, exchange_code: &str) -> Option<ExchangeRule> {
        self.exchanges.read().get(exchange_code).cloned()
    }

    pub fn is_holiday(&self, exchange_code: &str, date: NaiveDate) -> bool {
        self.holidays
            .read()
            .contains(&(exchange_code.to_string(), date))
    }
}

pub async fn hydrate(pool: &sqlx::PgPool, cache: &CalendarCache) {
    match load(pool).await {
        Ok((exchanges, symbols, holidays)) => {
            *cache.exchanges.write() = exchanges;
            *cache.symbols.write() = symbols;
            *cache.holidays.write() = holidays;
            info!(
                exchanges = cache.exchanges.read().len(),
                symbols = cache.symbols.read().len(),
                holidays = cache.holidays.read().len(),
                "hydrated market calendar cache"
            );
        }
        Err(err) => warn!(error = %err, "failed to hydrate market calendar cache"),
    }
}

pub async fn run_refresh(pool: sqlx::PgPool, cache: CalendarCache, refresh_sec: u64) {
    loop {
        hydrate(&pool, &cache).await;
        tokio::time::sleep(std::time::Duration::from_secs(refresh_sec)).await;
    }
}

type ExchangeRow = (
    String,
    String,
    String,
    Option<NaiveTime>,
    Option<NaiveTime>,
    Vec<String>,
);

type SymbolRow = (String, String, String);
type HolidayRow = (String, NaiveDate);

type LoadedCalendar = (
    HashMap<String, ExchangeRule>,
    HashMap<String, SymbolExchange>,
    HashSet<(String, NaiveDate)>,
);

async fn load(pool: &sqlx::PgPool) -> Result<LoadedCalendar, sqlx::Error> {
    let exchange_rows: Vec<ExchangeRow> = sqlx::query_as(
        "SELECT exchange_code, name, timezone, regular_open, regular_close, working_days FROM market.exchanges",
    )
    .fetch_all(pool)
    .await?;

    let symbol_rows: Vec<SymbolRow> =
        sqlx::query_as("SELECT symbol, exchange_code, asset_type FROM market.symbol_exchange_map")
            .fetch_all(pool)
            .await?;

    let holiday_rows: Vec<HolidayRow> = sqlx::query_as(
        "SELECT exchange_code, holiday_date FROM market.exchange_holidays WHERE is_open = FALSE",
    )
    .fetch_all(pool)
    .await?;

    let exchanges = exchange_rows
        .into_iter()
        .map(|row| {
            let code = row.0.to_uppercase();
            (
                code.clone(),
                ExchangeRule {
                    exchange_code: code,
                    name: row.1,
                    timezone: row.2,
                    regular_open: row.3,
                    regular_close: row.4,
                    working_days: row.5.into_iter().map(|day| day.to_lowercase()).collect(),
                },
            )
        })
        .collect();

    let symbols = symbol_rows
        .into_iter()
        .map(|row| {
            let symbol = row.0.to_uppercase();
            (
                symbol.clone(),
                SymbolExchange {
                    symbol,
                    exchange_code: row.1.to_uppercase(),
                    asset_type: row.2.to_lowercase(),
                },
            )
        })
        .collect();

    let holidays = holiday_rows
        .into_iter()
        .map(|row| (row.0.to_uppercase(), row.1))
        .collect();

    Ok((exchanges, symbols, holidays))
}
