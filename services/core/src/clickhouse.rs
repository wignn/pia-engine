use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ClickHouseClient {
    client: reqwest::Client,
    url: String,
    database: String,
    user: String,
    password: String,
}

#[derive(Clone, Debug)]
pub struct OhlcvCandle {
    pub symbol: String,
    pub resolution: String,
    pub time: chrono::DateTime<chrono::Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Clone, Debug)]
pub struct PriceTick {
    pub symbol: String,
    pub asset_type: String,
    pub source: String,
    pub time: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: f64,
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

struct RollupSpec {
    resolution: &'static str,
    table: &'static str,
    view: &'static str,
    bucket_expr: &'static str,
}

impl RollupSpec {
    fn new(resolution: &'static str, bucket_expr: &'static str) -> Self {
        Self {
            resolution,
            table: rollup_table(resolution).unwrap_or(""),
            view: match resolution {
                "5m" => "ohlcv_candles_5m_mv",
                "15m" => "ohlcv_candles_15m_mv",
                "1h" => "ohlcv_candles_1h_mv",
                _ => "",
            },
            bucket_expr,
        }
    }
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

    pub async fn bootstrap(&self) -> anyhow::Result<()> {
        self.execute(&format!(
            "CREATE DATABASE IF NOT EXISTS {}",
            ident(&self.database)
        ))
        .await?;
        self.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {}.ohlcv_candles (\
             symbol String, \
             resolution LowCardinality(String), \
             time DateTime64(3, 'UTC'), \
             open Float64, \
             high Float64, \
             low Float64, \
             close Float64, \
             volume Float64\
             ) ENGINE = MergeTree \
             PARTITION BY toYYYYMM(time) \
             ORDER BY (symbol, resolution, time)",
            ident(&self.database)
        ))
        .await?;
        self.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {}.price_ticks (\
             symbol String, \
             asset_type LowCardinality(String), \
             source LowCardinality(String), \
             time DateTime64(3, 'UTC'), \
             price Float64, \
             bid Nullable(Float64), \
             ask Nullable(Float64), \
             volume Float64\
             ) ENGINE = MergeTree \
             PARTITION BY toYYYYMMDD(time) \
             ORDER BY (symbol, time) \
             TTL time + INTERVAL 7 DAY",
            ident(&self.database)
        ))
        .await?;

        for rollup in [
            RollupSpec::new("5m", "toStartOfFiveMinutes(time)"),
            RollupSpec::new("15m", "toStartOfInterval(time, INTERVAL 15 MINUTE)"),
            RollupSpec::new("1h", "toStartOfHour(time)"),
        ] {
            self.create_rollup(&rollup).await?;
        }

        Ok(())
    }

    async fn create_rollup(&self, rollup: &RollupSpec) -> anyhow::Result<()> {
        let db = ident(&self.database);
        self.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {db}.{} (\
             symbol String, \
             resolution LowCardinality(String), \
             time DateTime64(3, 'UTC'), \
             open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')), \
             high_state AggregateFunction(max, Float64), \
             low_state AggregateFunction(min, Float64), \
             close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')), \
             volume_state AggregateFunction(sum, Float64)\
             ) ENGINE = AggregatingMergeTree \
             PARTITION BY toYYYYMM(time) \
             ORDER BY (symbol, resolution, time)",
            rollup.table
        ))
        .await?;

        self.execute(&format!(
            "CREATE MATERIALIZED VIEW IF NOT EXISTS {db}.{} TO {db}.{} AS \
             SELECT symbol, '{}' AS resolution, {} AS time, \
             argMinState(open, time) AS open_state, \
             maxState(high) AS high_state, \
             minState(low) AS low_state, \
             argMaxState(close, time) AS close_state, \
             sumState(volume) AS volume_state \
             FROM {db}.ohlcv_candles \
             WHERE resolution = '1m' \
             GROUP BY symbol, time",
            rollup.view, rollup.table, rollup.resolution, rollup.bucket_expr
        ))
        .await
    }

    pub async fn insert_candle(&self, candle: &OhlcvCandle) -> anyhow::Result<()> {
        let row = serde_json::json!({
            "symbol": candle.symbol,
            "resolution": candle.resolution,
            "time": candle.time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            "open": candle.open,
            "high": candle.high,
            "low": candle.low,
            "close": candle.close,
            "volume": candle.volume,
        });
        let sql = format!(
            "INSERT INTO {}.ohlcv_candles FORMAT JSONEachRow\n{}",
            ident(&self.database),
            row
        );
        self.execute(&sql).await
    }

    pub async fn insert_tick(&self, tick: &PriceTick) -> anyhow::Result<()> {
        let row = serde_json::json!({
            "symbol": tick.symbol,
            "asset_type": tick.asset_type,
            "source": tick.source,
            "time": tick.time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            "price": tick.price,
            "bid": tick.bid,
            "ask": tick.ask,
            "volume": tick.volume,
        });
        let sql = format!(
            "INSERT INTO {}.price_ticks FORMAT JSONEachRow\n{}",
            ident(&self.database),
            row
        );
        self.execute(&sql).await
    }

    pub async fn latest_history(
        &self,
        symbol: &str,
        resolution: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<Value>> {
        let sql = if resolution == "1m" {
            format!(
                "SELECT toUnixTimestamp(time) AS time, close AS value \
                 FROM {}.ohlcv_candles \
                 WHERE symbol = {} AND resolution = '1m' \
                 ORDER BY time DESC \
                 LIMIT {} \
                 FORMAT JSONEachRow",
                ident(&self.database),
                string_literal(symbol),
                limit.clamp(1, 1000)
            )
        } else {
            let table = rollup_table(resolution).ok_or_else(|| {
                anyhow::anyhow!("unsupported ClickHouse history resolution: {resolution}")
            })?;
            format!(
                "SELECT toUnixTimestamp(time) AS time, argMaxMerge(close_state) AS value \
                 FROM {}.{} \
                 WHERE symbol = {} AND resolution = {} \
                 GROUP BY symbol, resolution, time \
                 ORDER BY time DESC \
                 LIMIT {} \
                 FORMAT JSONEachRow",
                ident(&self.database),
                table,
                string_literal(symbol),
                string_literal(resolution),
                limit.clamp(1, 1000)
            )
        };
        let text = self.query(&sql).await?;
        let mut rows: Vec<Value> = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect();
        rows.reverse();
        Ok(rows)
    }

    pub async fn latest_prices(&self) -> anyhow::Result<Vec<LatestPriceTick>> {
        let sql = format!(
            "SELECT symbol, \
             argMax(price, time) AS price, \
             argMax(bid, time) AS bid, \
             argMax(ask, time) AS ask, \
             argMax(volume, time) AS volume, \
             argMax(source, time) AS source, \
             argMax(asset_type, time) AS asset_type, \
             formatDateTime(max(time), '%Y-%m-%dT%H:%i:%SZ') AS received_at \
             FROM {}.price_ticks \
             GROUP BY symbol \
             ORDER BY symbol \
             FORMAT JSONEachRow",
            ident(&self.database)
        );
        let text = self.query(&sql).await?;
        Ok(text
            .lines()
            .filter_map(|line| serde_json::from_str::<LatestPriceTick>(line).ok())
            .collect())
    }

    pub async fn latest_price(&self, symbol: &str) -> anyhow::Result<Option<LatestPriceTick>> {
        Ok(self
            .latest_prices()
            .await?
            .into_iter()
            .find(|price| price.symbol.eq_ignore_ascii_case(symbol)))
    }

    pub async fn tick_stats(&self) -> anyhow::Result<Vec<Value>> {
        let sql = format!(
            "SELECT symbol, asset_type, source, \
             toUnixTimestamp(max(time)) AS last_tick_time, \
             anyLast(price) AS latest_price, \
             countIf(time >= now() - INTERVAL 5 MINUTE) AS ticks_5m, \
             countIf(time >= now() - INTERVAL 1 HOUR) AS ticks_1h, \
             uniqExact(price) AS unique_prices_1h \
             FROM {}.price_ticks \
             WHERE time >= now() - INTERVAL 1 HOUR \
             GROUP BY symbol, asset_type, source \
             FORMAT JSONEachRow",
            ident(&self.database)
        );
        let text = self.query(&sql).await?;
        Ok(text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect())
    }

    async fn execute(&self, sql: &str) -> anyhow::Result<()> {
        self.request(sql).await.map(|_| ())
    }

    async fn query(&self, sql: &str) -> anyhow::Result<String> {
        self.request(sql).await
    }

    async fn request(&self, sql: &str) -> anyhow::Result<String> {
        let mut req = self.client.post(&self.url).body(sql.to_string());
        if !self.user.is_empty() {
            req = req.basic_auth(&self.user, Some(&self.password));
        }
        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            anyhow::bail!("ClickHouse HTTP error {status}: {text}");
        }
        Ok(text)
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
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_identifiers_and_literals() {
        assert_eq!(ident("market-prod;DROP"), "marketprodDROP");
        assert_eq!(string_literal("XAU'USD"), "'XAU''USD'");
    }

    #[test]
    fn maps_rollup_tables() {
        assert_eq!(rollup_table("5m"), Some("ohlcv_candles_5m"));
        assert_eq!(rollup_table("15m"), Some("ohlcv_candles_15m"));
        assert_eq!(rollup_table("1h"), Some("ohlcv_candles_1h"));
        assert_eq!(rollup_table("1d"), None);
    }

    #[test]
    fn parses_latest_price_tick_row() {
        let row: LatestPriceTick = serde_json::from_str(
            r#"{"symbol":"XAUUSD","price":4500.5,"bid":4500.1,"ask":4500.9,"volume":0,"source":"market_data","asset_type":"forex","received_at":"2026-05-24T12:00:00Z"}"#,
        )
        .unwrap();

        assert_eq!(row.symbol, "XAUUSD");
        assert_eq!(row.price, 4500.5);
        assert_eq!(row.asset_type, "forex");
    }
}
