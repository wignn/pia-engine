use serde::Deserialize;

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
        let threshold = 0.0;
        let sql = format!(
            "WITH latest AS (SELECT symbol, argMax(asset_type, time) AS asset_type, argMax(price, time) AS latest_price, max(time) AS latest_at FROM {}.price_ticks GROUP BY symbol), baseline AS (SELECT symbol, argMin(price, time) AS baseline_price, count() AS tick_count FROM {}.price_ticks WHERE time >= now() - INTERVAL {} MINUTE GROUP BY symbol) SELECT latest.symbol, latest.asset_type, latest.latest_price, baseline.baseline_price, ((latest.latest_price - baseline.baseline_price) / baseline.baseline_price) * 100 AS move_pct, baseline.tick_count, toString(latest.latest_at) AS latest_at FROM latest INNER JOIN baseline ON latest.symbol = baseline.symbol WHERE baseline.baseline_price > 0 AND abs(move_pct) >= {} ORDER BY abs(move_pct) DESC LIMIT 100 FORMAT JSONEachRow",
            ident(&self.database),
            ident(&self.database),
            window_minutes.clamp(1, 240),
            threshold,
        );
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

fn ident(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
}
