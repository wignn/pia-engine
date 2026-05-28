use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub struct ClickHouseClient {
    client: reqwest::Client,
    url: String,
    database: String,
    user: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpikeCandidate {
    pub symbol: String,
    pub asset_type: String,
    pub latest_price: f64,
    pub baseline_price: f64,
    pub move_pct: f64,
    #[serde(deserialize_with = "deserialize_u64")]
    pub tick_count: u64,
    pub latest_at: String,
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

    pub async fn spike_candidates(
        &self,
        window_minutes: u32,
    ) -> anyhow::Result<Vec<SpikeCandidate>> {
        let sql = spike_candidates_sql(&self.database, window_minutes);
        let text = self.query(&sql).await?;
        Ok(text
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
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
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("invalid u64 number")),
        serde_json::Value::String(value) => value.parse().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("expected u64 number or string")),
    }
}

fn ident(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
}

fn spike_candidates_sql(database: &str, window_minutes: u32) -> String {
    let threshold = 0.0;
    let bounded_minutes = window_minutes.clamp(1, 240).max(60);
    format!(
        "WITH recent AS (SELECT symbol, asset_type, price, time FROM {}.price_ticks WHERE time >= now() - INTERVAL {} MINUTE), latest AS (SELECT symbol, argMax(asset_type, time) AS asset_type, argMax(price, time) AS latest_price, max(time) AS latest_at FROM recent GROUP BY symbol), baseline AS (SELECT symbol, argMin(price, time) AS baseline_price, count() AS tick_count FROM recent GROUP BY symbol) SELECT latest.symbol, latest.asset_type, latest.latest_price, baseline.baseline_price, ((latest.latest_price - baseline.baseline_price) / baseline.baseline_price) * 100 AS move_pct, baseline.tick_count, toString(latest.latest_at) AS latest_at FROM latest INNER JOIN baseline ON latest.symbol = baseline.symbol WHERE baseline.baseline_price > 0 AND abs(move_pct) >= {} ORDER BY abs(move_pct) DESC LIMIT 100 FORMAT JSONEachRow",
        ident(database),
        bounded_minutes,
        threshold,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spike_candidates_query_uses_bounded_context_window() {
        let sql = spike_candidates_sql("market", 5);

        assert!(sql.contains("WITH recent AS"));
        assert!(sql.contains("WHERE time >= now() - INTERVAL 60 MINUTE"));
        assert!(sql.contains("FROM recent GROUP BY symbol"));
        assert!(!sql.contains("FROM market.price_ticks GROUP BY symbol"));
    }

    #[test]
    fn spike_candidates_query_clamps_large_context_windows() {
        let sql = spike_candidates_sql("market", 240);

        assert!(sql.contains("WHERE time >= now() - INTERVAL 240 MINUTE"));
        assert!(sql.contains("LIMIT 100"));
    }

    #[test]
    fn spike_candidate_accepts_clickhouse_uint64_string_fields() {
        let row = serde_json::from_str::<SpikeCandidate>(
            r#"{"symbol":"XAUUSD","asset_type":"forex","latest_price":4450.02,"baseline_price":4449,"move_pct":0.0229,"tick_count":"15","latest_at":"2026-05-27 14:19:54.305"}"#,
        )
        .unwrap();

        assert_eq!(row.tick_count, 15);
    }
}
