use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::prices::CachedPrice;

#[derive(Clone, Debug)]
pub struct ClickHouseClient {
    client: reqwest::Client,
    url: String,
    database: String,
    user: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LatestPriceTick {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: f64,
    pub source: String,
    pub asset_type: String,
    pub received_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpikeCandidate {
    pub symbol: String,
    pub asset_type: String,
    pub latest_price: f64,
    pub baseline_price: f64,
    pub move_pct: f64,
    pub tick_count: u64,
    pub latest_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TickStats {
    pub symbol: String,
    pub ticks_5m: u64,
    pub latest_at: String,
}

#[derive(Serialize)]
struct PriceTickRow<'a> {
    symbol: &'a str,
    time: String,
    price: f64,
    bid: Option<f64>,
    ask: Option<f64>,
    volume: f64,
    source: &'a str,
    asset_type: &'a str,
}

#[derive(Serialize)]
struct OhlcvCandleRow<'a> {
    symbol: &'a str,
    resolution: &'static str,
    time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl ClickHouseClient {
    pub fn new(url: String, database: String, user: String, password: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.trim_end_matches('/').to_string(),
            database,
            user,
            password,
        }
    }

    pub async fn insert_price_tick(
        &self,
        price: &CachedPrice,
        received_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "INSERT INTO {}.price_ticks (symbol, time, price, bid, ask, volume, source, asset_type) VALUES ({}, {}, {}, {}, {}, {}, {}, {})",
            ident(&self.database),
            string_literal(&price.symbol),
            datetime_literal(received_at),
            price.price,
            nullable_f64(price.bid),
            nullable_f64(price.ask),
            price.volume.unwrap_or(0.0),
            string_literal(&price.source),
            string_literal(&price.asset_type),
        );
        self.query(&sql).await?;
        Ok(())
    }

    pub async fn insert_ohlcv_candle(
        &self,
        price: &CachedPrice,
        minute: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let volume = price.volume.unwrap_or(0.0);
        let sql = format!(
            "INSERT INTO {}.ohlcv_candles (symbol, resolution, time, open, high, low, close, volume, updated_at) VALUES ({}, '1m', {}, {}, {}, {}, {}, {}, now64(3))",
            ident(&self.database),
            string_literal(&price.symbol),
            datetime_literal(minute),
            price.price,
            price.price,
            price.price,
            price.price,
            volume,
        );
        self.query(&sql).await?;
        Ok(())
    }

    pub async fn insert_price_ticks_batch(
        &self,
        batch: &[(CachedPrice, DateTime<Utc>)],
    ) -> anyhow::Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut body = String::new();
        for (price, received_at) in batch {
            let row = PriceTickRow {
                symbol: &price.symbol,
                time: received_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                price: price.price,
                bid: price.bid,
                ask: price.ask,
                volume: price.volume.unwrap_or(0.0),
                source: &price.source,
                asset_type: &price.asset_type,
            };
            body.push_str(&serde_json::to_string(&row)?);
            body.push('\n');
        }

        let sql = format!(
            "INSERT INTO {}.price_ticks FORMAT JSONEachRow",
            ident(&self.database)
        );
        self.insert_json_each_row(&sql, body, "ClickHouse price tick batch insert")
            .await
    }

    pub async fn insert_ohlcv_candles_batch(
        &self,
        batch: &[(CachedPrice, DateTime<Utc>)],
    ) -> anyhow::Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut body = String::new();
        for (price, minute) in batch {
            let row = OhlcvCandleRow {
                symbol: &price.symbol,
                resolution: "1m",
                time: minute.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                open: price.price,
                high: price.price,
                low: price.price,
                close: price.price,
                volume: price.volume.unwrap_or(0.0),
            };
            body.push_str(&serde_json::to_string(&row)?);
            body.push('\n');
        }

        let sql = format!(
            "INSERT INTO {}.ohlcv_candles FORMAT JSONEachRow",
            ident(&self.database)
        );
        self.insert_json_each_row(&sql, body, "ClickHouse ohlcv candle batch insert")
            .await
    }

    pub async fn latest_prices(&self) -> anyhow::Result<Vec<LatestPriceTick>> {
        let sql = format!(
            "SELECT symbol, argMax(price, time) AS price, argMax(bid, time) AS bid, argMax(ask, time) AS ask, argMax(volume, time) AS volume, argMax(source, time) AS source, argMax(asset_type, time) AS asset_type, toString(max(time)) AS received_at FROM {}.price_ticks GROUP BY symbol ORDER BY symbol FORMAT JSONEachRow",
            ident(&self.database)
        );
        self.query_json_each_row(&sql).await
    }

    pub async fn latest_price(&self, symbol: &str) -> anyhow::Result<Option<LatestPriceTick>> {
        let sql = format!(
            "SELECT symbol, argMax(price, time) AS price, argMax(bid, time) AS bid, argMax(ask, time) AS ask, argMax(volume, time) AS volume, argMax(source, time) AS source, argMax(asset_type, time) AS asset_type, toString(max(time)) AS received_at FROM {}.price_ticks WHERE symbol = {} GROUP BY symbol FORMAT JSONEachRow",
            ident(&self.database),
            string_literal(symbol)
        );
        Ok(self.query_json_each_row(&sql).await?.into_iter().next())
    }

    pub async fn latest_history(
        &self,
        symbol: &str,
        resolution: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<Value>> {
        let sql = if resolution == "1m" {
            format!(
                "SELECT toUnixTimestamp(time) AS time, close AS value FROM {}.ohlcv_candles WHERE symbol = {} AND resolution = '1m' ORDER BY time DESC LIMIT {} FORMAT JSONEachRow",
                ident(&self.database),
                string_literal(symbol),
                limit.clamp(1, 1000)
            )
        } else {
            let table = rollup_table(resolution).ok_or_else(|| {
                anyhow::anyhow!("unsupported ClickHouse history resolution: {resolution}")
            })?;
            format!(
                "SELECT toUnixTimestamp(time) AS time, argMaxMerge(close_state) AS value FROM {}.{} WHERE symbol = {} AND resolution = {} GROUP BY symbol, resolution, time ORDER BY time DESC LIMIT {} FORMAT JSONEachRow",
                ident(&self.database),
                table,
                string_literal(symbol),
                string_literal(resolution),
                limit.clamp(1, 1000)
            )
        };
        let mut rows: Vec<Value> = self.query_json_each_row(&sql).await?;
        rows.reverse();
        Ok(rows)
    }

    pub async fn spike_candidates(
        &self,
        window_minutes: u32,
        threshold_pct: f64,
        limit: usize,
    ) -> anyhow::Result<Vec<SpikeCandidate>> {
        let sql = format!(
            "WITH latest AS (SELECT symbol, argMax(asset_type, time) AS asset_type, argMax(price, time) AS latest_price, max(time) AS latest_at FROM {}.price_ticks GROUP BY symbol), baseline AS (SELECT symbol, argMin(price, time) AS baseline_price, count() AS tick_count FROM {}.price_ticks WHERE time >= now() - INTERVAL {} MINUTE GROUP BY symbol) SELECT latest.symbol, latest.asset_type, latest.latest_price, baseline.baseline_price, ((latest.latest_price - baseline.baseline_price) / baseline.baseline_price) * 100 AS move_pct, baseline.tick_count, toString(latest.latest_at) AS latest_at FROM latest INNER JOIN baseline ON latest.symbol = baseline.symbol WHERE baseline.baseline_price > 0 AND abs(move_pct) >= {} ORDER BY abs(move_pct) DESC LIMIT {} FORMAT JSONEachRow",
            ident(&self.database),
            ident(&self.database),
            window_minutes.clamp(1, 240),
            threshold_pct,
            limit.clamp(1, 100)
        );
        self.query_json_each_row(&sql).await
    }

    pub async fn tick_stats(&self) -> anyhow::Result<Vec<TickStats>> {
        let sql = format!(
            "SELECT symbol, countIf(time >= now() - INTERVAL 5 MINUTE) AS ticks_5m, toString(max(time)) AS latest_at FROM {}.price_ticks GROUP BY symbol FORMAT JSONEachRow",
            ident(&self.database)
        );
        self.query_json_each_row(&sql).await
    }

    async fn query_json_each_row<T>(&self, sql: &str) -> anyhow::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let text = self.query(sql).await?;
        Ok(text
            .lines()
            .filter_map(|line| serde_json::from_str::<T>(line).ok())
            .collect())
    }

    async fn query(&self, sql: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .post(&self.url)
            .basic_auth(&self.user, Some(&self.password))
            .query(&[("database", &self.database)])
            .body(sql.to_string())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("ClickHouse query failed with {status}: {text}");
        }
        Ok(text)
    }

    async fn insert_json_each_row(
        &self,
        sql: &str,
        body: String,
        operation: &str,
    ) -> anyhow::Result<()> {
        let response = self
            .client
            .post(&self.url)
            .basic_auth(&self.user, Some(&self.password))
            .query(&[("database", self.database.as_str()), ("query", sql)])
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("{operation} failed with {status}: {text}");
        }
        Ok(())
    }
}

fn rollup_table(resolution: &str) -> Option<&'static str> {
    match resolution {
        "5m" => Some("ohlcv_candles_5m"),
        "15m" => Some("ohlcv_candles_15m"),
        "1h" => Some("ohlcv_candles_1h"),
        _ => None,
    }
}

fn ident(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
}

fn string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\''"))
}

fn datetime_literal(value: DateTime<Utc>) -> String {
    format!(
        "toDateTime64('{}', 3, 'UTC')",
        value.format("%Y-%m-%d %H:%M:%S%.3f")
    )
}

fn nullable_f64(value: Option<f64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollup_table_maps_supported_resolutions() {
        assert_eq!(rollup_table("5m"), Some("ohlcv_candles_5m"));
        assert_eq!(rollup_table("1h"), Some("ohlcv_candles_1h"));
        assert_eq!(rollup_table("1d"), None);
    }

    #[test]
    fn test_jsoneachrow_serialization() {
        let price = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            bid: Some(49990.0),
            ask: Some(50010.0),
            volume: Some(1.5),
            source: "test".to_string(),
            asset_type: "crypto".to_string(),
            received_at: Some("2026-05-28T12:00:00Z".to_string()),
        };

        let received_at = chrono::Utc::now();
        let payload = serde_json::json!({
            "symbol": price.symbol,
            "time": received_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            "price": price.price,
            "bid": price.bid,
            "ask": price.ask,
            "volume": price.volume.unwrap_or(0.0),
            "source": price.source,
            "asset_type": price.asset_type,
        });

        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains("\"symbol\":\"BTCUSDT\""));
        assert!(serialized.contains("\"price\":50000.0"));
    }
}
