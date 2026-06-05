use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

fn macro_geosignal_event_id(series_id: &str, signal_date: NaiveDate) -> String {
    format!("macro:{series_id}:{signal_date}")
}

fn macro_severity_score(severity: &str) -> f64 {
    match severity.trim().to_lowercase().as_str() {
        "high" => 0.75,
        "medium" => 0.5,
        "low" => 0.25,
        _ => 0.25,
    }
}

fn macro_sentiment_score(direction: &str) -> f64 {
    match direction.trim().to_lowercase().as_str() {
        "up" => 0.25,
        "down" => -0.25,
        _ => 0.0,
    }
}

fn macro_affected_assets(category: &str, series_id: &str) -> Vec<String> {
    match (category, series_id) {
        ("rates", "DGS10") => vec!["US10Y".to_string()],
        ("rates", "DGS2") => vec!["US02Y".to_string()],
        ("dollar_liquidity", _) => vec!["DXY".to_string()],
        ("commodities", "DCOILWTICO") => vec!["WTI".to_string()],
        ("commodities", "DCOILBRENTEU") => vec!["BRENT".to_string()],
        _ => Vec::new(),
    }
}

fn macro_asset_impact(assets: &[String], severity_score: f64) -> Value {
    Value::Object(
        assets
            .iter()
            .map(|asset| (asset.clone(), json!(severity_score)))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builds_macro_geosignal_event_id() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();
        assert_eq!(
            macro_geosignal_event_id("DGS10", date),
            "macro:DGS10:2026-06-05"
        );
    }

    #[test]
    fn maps_macro_severity_to_score() {
        assert_eq!(macro_severity_score("low"), 0.25);
        assert_eq!(macro_severity_score("medium"), 0.5);
        assert_eq!(macro_severity_score("high"), 0.75);
        assert_eq!(macro_severity_score("unknown"), 0.25);
    }

    #[test]
    fn maps_macro_direction_to_sentiment_score() {
        assert_eq!(macro_sentiment_score("up"), 0.25);
        assert_eq!(macro_sentiment_score("down"), -0.25);
        assert_eq!(macro_sentiment_score("flat"), 0.0);
        assert_eq!(macro_sentiment_score("unknown"), 0.0);
    }

    #[test]
    fn maps_macro_category_to_affected_assets() {
        assert_eq!(macro_affected_assets("rates", "DGS10"), vec!["US10Y"]);
        assert_eq!(
            macro_affected_assets("dollar_liquidity", "DTWEXBGS"),
            vec!["DXY"]
        );
        assert_eq!(
            macro_affected_assets("commodities", "DCOILWTICO"),
            vec!["WTI"]
        );
        assert_eq!(
            macro_affected_assets("commodities", "DCOILBRENTEU"),
            vec!["BRENT"]
        );
        assert!(macro_affected_assets("growth", "GDP").is_empty());
    }
}

#[derive(Debug)]
struct MacroPoint {
    date: NaiveDate,
    value: f64,
}

pub async fn refresh_signals(pool: &PgPool) -> anyhow::Result<usize> {
    let series = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, title, category FROM macro_series WHERE provider = 'fred' ORDER BY id",
    )
    .fetch_all(pool)
    .await?;

    let mut upserted = 0usize;
    for (series_id, title, category) in series {
        let points = load_recent_points(pool, &series_id).await?;
        if points.len() < 2 {
            continue;
        }
        let latest = &points[0];
        let previous = &points[1];
        let change_1d = latest.value - previous.value;
        let change_7d = nearest_change(&points, latest.date, 7);
        let change_30d = nearest_change(&points, latest.date, 30);
        let direction = direction(change_1d);
        let severity = severity(&series_id, change_1d, change_7d);
        let narrative = narrative(
            &series_id,
            &title,
            &category,
            latest.value,
            change_1d,
            change_7d,
        );

        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO macro_signals (series_id, category, signal_date, latest_value, previous_value, change_1d, change_7d, change_30d, direction, severity, narrative)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (series_id, signal_date) DO UPDATE SET category = EXCLUDED.category, latest_value = EXCLUDED.latest_value, previous_value = EXCLUDED.previous_value, change_1d = EXCLUDED.change_1d, change_7d = EXCLUDED.change_7d, change_30d = EXCLUDED.change_30d, direction = EXCLUDED.direction, severity = EXCLUDED.severity, narrative = EXCLUDED.narrative, updated_at = NOW()",
        )
        .bind(&series_id)
        .bind(&category)
        .bind(latest.date)
        .bind(latest.value)
        .bind(previous.value)
        .bind(change_1d)
        .bind(change_7d)
        .bind(change_30d)
        .bind(direction)
        .bind(severity)
        .bind(&narrative)
        .execute(&mut *tx)
        .await?;

        let assets = macro_affected_assets(&category, &series_id);
        let severity_score = macro_severity_score(severity);
        let timestamp = Utc
            .with_ymd_and_hms(
                latest.date.year(),
                latest.date.month(),
                latest.date.day(),
                0,
                0,
                0,
            )
            .single()
            .unwrap_or_else(Utc::now);
        sqlx::query(
            "INSERT INTO news.geosignals (event_id, timestamp, source, source_url, title, summary, category, country, region, location_scope, severity_score, sentiment_score, confidence_score, affected_assets, asset_impact, freshness)
             VALUES ($1, $2, 'fred', NULL, $3, $4, 'macro', NULL, NULL, 'global', $5, $6, 0.75, $7, $8, 'fresh')
             ON CONFLICT (event_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, title = EXCLUDED.title, summary = EXCLUDED.summary, severity_score = EXCLUDED.severity_score, sentiment_score = EXCLUDED.sentiment_score, affected_assets = EXCLUDED.affected_assets, asset_impact = EXCLUDED.asset_impact, freshness = EXCLUDED.freshness",
        )
        .bind(macro_geosignal_event_id(&series_id, latest.date))
        .bind(timestamp)
        .bind(&title)
        .bind(&narrative)
        .bind(severity_score)
        .bind(macro_sentiment_score(direction))
        .bind(&assets)
        .bind(macro_asset_impact(&assets, severity_score))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        upserted += 1;
    }

    Ok(upserted)
}

pub async fn dashboard(pool: &PgPool, limit: i64) -> anyhow::Result<Value> {
    let rows = sqlx::query(
        "WITH latest AS (
            SELECT DISTINCT ON (s.id) s.id, ms.title, ms.category, s.signal_date, s.latest_value, s.change_1d, s.change_7d, s.change_30d, s.direction, s.severity, s.narrative
            FROM macro_signals s
            JOIN macro_series ms ON ms.id = s.series_id
            ORDER BY s.id, s.signal_date DESC
        )
        SELECT * FROM latest ORDER BY
            CASE severity WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END,
            category ASC,
            id ASC
        LIMIT $1",
    )
    .bind(limit.clamp(1, 100))
    .fetch_all(pool)
    .await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "series_id": row.get::<String, _>("id"),
                "title": row.get::<String, _>("title"),
                "category": row.get::<String, _>("category"),
                "signal_date": row.get::<NaiveDate, _>("signal_date"),
                "latest_value": row.get::<Option<f64>, _>("latest_value"),
                "change_1d": row.get::<Option<f64>, _>("change_1d"),
                "change_7d": row.get::<Option<f64>, _>("change_7d"),
                "change_30d": row.get::<Option<f64>, _>("change_30d"),
                "direction": row.get::<String, _>("direction"),
                "severity": row.get::<String, _>("severity"),
                "narrative": row.get::<String, _>("narrative"),
            })
        })
        .collect();

    Ok(json!({ "items": items, "total": items.len(), "source": "fred" }))
}

async fn load_recent_points(pool: &PgPool, series_id: &str) -> anyhow::Result<Vec<MacroPoint>> {
    let rows = sqlx::query_as::<_, (NaiveDate, f64)>(
        "SELECT observation_date, value FROM macro_observations WHERE series_id = $1 AND value IS NOT NULL ORDER BY observation_date DESC LIMIT 90",
    )
    .bind(series_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(date, value)| MacroPoint { date, value })
        .collect())
}

fn nearest_change(points: &[MacroPoint], latest_date: NaiveDate, days: i64) -> Option<f64> {
    let target = latest_date - chrono::Duration::days(days);
    points
        .iter()
        .filter(|point| point.date <= target)
        .min_by_key(|point| (target - point.date).num_days().abs())
        .map(|point| points[0].value - point.value)
}

fn direction(change_1d: f64) -> &'static str {
    if change_1d > 0.0 {
        "up"
    } else if change_1d < 0.0 {
        "down"
    } else {
        "flat"
    }
}

fn severity(series_id: &str, change_1d: f64, change_7d: Option<f64>) -> &'static str {
    let weekly = change_7d.unwrap_or(change_1d).abs();
    let daily = change_1d.abs();
    let threshold = match series_id {
        "DGS2" | "DGS10" | "DGS30" | "T10Y2Y" | "T5YIE" | "T10YIE" => 0.10,
        "DCOILWTICO" | "DCOILBRENTEU" => 3.0,
        "BAMLH0A0HYM2" | "BAMLC0A0CM" => 0.20,
        "ICSA" => 10_000.0,
        _ => 0.5,
    };

    if daily >= threshold || weekly >= threshold * 2.0 {
        "high"
    } else if daily >= threshold / 2.0 || weekly >= threshold {
        "medium"
    } else {
        "low"
    }
}

fn narrative(
    series_id: &str,
    title: &str,
    category: &str,
    latest: f64,
    change_1d: f64,
    change_7d: Option<f64>,
) -> String {
    let bias = match (series_id, change_1d.is_sign_positive()) {
        ("DGS10" | "DGS2", true) => "higher yields can pressure gold and risk assets",
        ("DGS10" | "DGS2", false) => "lower yields can support gold and liquidity-sensitive assets",
        ("DTWEXBGS", true) => "a stronger dollar can pressure commodities and non-USD assets",
        ("DTWEXBGS", false) => "a softer dollar can support gold and risk appetite",
        ("BAMLH0A0HYM2" | "BAMLC0A0CM", true) => "wider credit spreads indicate rising stress",
        ("BAMLH0A0HYM2" | "BAMLC0A0CM", false) => {
            "tighter credit spreads indicate calmer risk conditions"
        }
        ("DCOILWTICO" | "DCOILBRENTEU", true) => "higher oil can add inflation pressure",
        ("DCOILWTICO" | "DCOILBRENTEU", false) => "lower oil can ease inflation pressure",
        _ => "macro context updated",
    };
    let weekly = change_7d
        .map(|value| format!(", 7d change {value:.2}"))
        .unwrap_or_default();
    format!("{title} ({category}) is at {latest:.2}, 1d change {change_1d:.2}{weekly}; {bias}.")
}
