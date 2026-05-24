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

    pub async fn latest_history(&self, symbol: &str, limit: usize) -> anyhow::Result<Vec<Value>> {
        let sql = format!(
            "SELECT toUnixTimestamp(time) AS time, close AS value \
             FROM {}.ohlcv_candles \
             WHERE symbol = {} AND resolution = '1m' \
             ORDER BY time DESC \
             LIMIT {} \
             FORMAT JSONEachRow",
            ident(&self.database),
            string_literal(symbol),
            limit.clamp(1, 1000)
        );
        let text = self.query(&sql).await?;
        let mut rows: Vec<Value> = text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect();
        rows.reverse();
        Ok(rows)
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
}
