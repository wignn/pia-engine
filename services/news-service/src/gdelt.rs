use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Debug, Deserialize, Serialize)]
pub struct GdeltArticle {
    pub url: Option<String>,
    pub url_mobile: Option<String>,
    pub title: Option<String>,
    pub seendate: Option<String>,
    pub socialimage: Option<String>,
    pub domain: Option<String>,
    pub language: Option<String>,
    pub sourcecountry: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GdeltDocResponse {
    pub articles: Option<Vec<GdeltArticle>>,
}

pub fn map_assets(category: &str, context: &str) -> Vec<String> {
    let combined = format!("{} {}", category, context).to_lowercase();
    if combined.contains("conflict")
        || combined.contains("geopolitical")
        || combined.contains("war")
        || combined.contains("strike")
        || combined.contains("military")
        || combined.contains("middle east")
    {
        vec!["XAUUSD".to_string(), "DXY".to_string()]
    } else if combined.contains("energy") || combined.contains("oil") || combined.contains("crude")
    {
        vec!["WTI".to_string(), "BRENT".to_string()]
    } else if combined.contains("tariff") || combined.contains("trade") {
        vec![
            "DXY".to_string(),
            "EURUSD".to_string(),
            "USDCNH".to_string(),
        ]
    } else {
        vec!["XAUUSD".to_string(), "DXY".to_string()]
    }
}

pub fn parse_seendate(date_str: Option<&str>) -> chrono::DateTime<chrono::Utc> {
    let s = match date_str {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return chrono::Utc::now(),
    };

    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ") {
        return dt.and_utc();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S") {
        return dt.and_utc();
    }

    chrono::Utc::now()
}

pub fn categorize_gdelt_article(title: &str) -> (String, f64, f64) {
    let lower = title.to_lowercase();
    if lower.contains("war")
        || lower.contains("conflict")
        || lower.contains("attack")
        || lower.contains("strike")
        || lower.contains("military")
        || lower.contains("missile")
    {
        ("geopolitical_conflict".to_string(), 0.85, -0.6)
    } else if lower.contains("sanction") || lower.contains("tariff") || lower.contains("trade war")
    {
        ("trade_dispute".to_string(), 0.70, -0.4)
    } else if lower.contains("election")
        || lower.contains("protest")
        || lower.contains("coup")
        || lower.contains("unrest")
    {
        ("political_unrest".to_string(), 0.60, -0.3)
    } else if lower.contains("earthquake")
        || lower.contains("flood")
        || lower.contains("hurricane")
        || lower.contains("tsunami")
    {
        ("natural_disaster".to_string(), 0.75, -0.5)
    } else {
        ("geopolitical".to_string(), 0.50, -0.1)
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn run_gdelt_sync(cfg: Config, pool: sqlx::PgPool) {
    let client = match reqwest::Client::builder()
        .user_agent("ATLSD/1.0 (news-service gdelt fetcher)")
        .timeout(Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            warn!(error = %err, "failed to build HTTP client for GDELT sync");
            return;
        }
    };

    info!("Starting GDELT geosignals sync loop");

    loop {
        if let Err(err) = sync_gdelt_data(&client, &pool).await {
            warn!(error = %err, "GDELT sync iteration failed");
        }

        tokio::time::sleep(Duration::from_secs(cfg.gdelt_sync_sec)).await;
    }
}

pub async fn sync_gdelt_data(client: &reqwest::Client, pool: &sqlx::PgPool) -> Result<(), String> {
    let gdelt_url = "https://api.gdeltproject.org/api/v2/doc/doc?query=geopolitical%20OR%20conflict%20OR%20sanctions%20OR%20war%20OR%20election&mode=ArtList&format=json&maxrecords=75";

    let resp = match client.get(gdelt_url).send().await {
        Ok(r) => r,
        Err(err) => return Err(format!("Failed to fetch GDELT DOC API: {}", err)),
    };

    if !resp.status().is_success() {
        return Err(format!(
            "GDELT DOC API returned status HTTP {}",
            resp.status()
        ));
    }

    let doc_resp: GdeltDocResponse = match resp.json().await {
        Ok(data) => data,
        Err(err) => return Err(format!("Failed to parse GDELT JSON response: {}", err)),
    };

    let articles = match doc_resp.articles {
        Some(arts) if !arts.is_empty() => arts,
        _ => return Ok(()),
    };

    let mut ingested = 0;

    for article in articles {
        let url = match &article.url {
            Some(u) if !u.trim().is_empty() => u.trim(),
            _ => continue,
        };

        let title = match &article.title {
            Some(t) if !t.trim().is_empty() => t.trim(),
            _ => continue,
        };

        let event_id = format!("gdelt_{}", &sha256_hex(url)[..32]);
        let raw_json = match serde_json::to_value(&article) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 1. Upsert raw event
        let raw_res = sqlx::query(
            r#"
            INSERT INTO geosignal_raw_events (source, event_id, fetched_at, raw_json)
            VALUES ('gdelt', $1, NOW(), $2)
            ON CONFLICT (source, event_id)
            DO UPDATE SET raw_json = EXCLUDED.raw_json, fetched_at = NOW()
            "#,
        )
        .bind(&event_id)
        .bind(&raw_json)
        .execute(pool)
        .await;

        if let Err(err) = raw_res {
            warn!(error = %err, event_id = %event_id, "failed to insert geosignal_raw_event");
            continue;
        }

        // 2. Normalize and upsert into news.geosignals
        let timestamp = parse_seendate(article.seendate.as_deref());
        let country = article
            .sourcecountry
            .as_deref()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        let (category, severity_score, sentiment_score) = categorize_gdelt_article(title);

        let location_scope = if country.is_some() {
            "country".to_string()
        } else {
            "global".to_string()
        };

        let affected_assets = map_assets(&category, country.as_deref().unwrap_or(title));
        let asset_impact = serde_json::json!({
            "affected_assets": affected_assets,
            "category": category,
            "severity": severity_score,
        });

        let geo_res = sqlx::query(
            r#"
            INSERT INTO news.geosignals (
                event_id, timestamp, source, source_url, title, summary, category, country, region,
                location_scope, severity_score, sentiment_score, confidence_score, affected_assets,
                asset_impact, freshness, created_at
            ) VALUES (
                $1, $2, 'gdelt', $3, $4, $5, $6, $7, NULL, $8, $9, $10, 0.8, $11, $12, 'fresh', NOW()
            )
            ON CONFLICT (event_id) DO UPDATE SET
                title = EXCLUDED.title,
                summary = EXCLUDED.summary,
                severity_score = EXCLUDED.severity_score,
                sentiment_score = EXCLUDED.sentiment_score,
                affected_assets = EXCLUDED.affected_assets,
                asset_impact = EXCLUDED.asset_impact,
                freshness = EXCLUDED.freshness
            "#,
        )
        .bind(&event_id)
        .bind(timestamp)
        .bind(url)
        .bind(title)
        .bind(title)
        .bind(&category)
        .bind(&country)
        .bind(&location_scope)
        .bind(severity_score)
        .bind(sentiment_score)
        .bind(&affected_assets)
        .bind(&asset_impact)
        .execute(pool)
        .await;

        if let Err(err) = geo_res {
            warn!(error = %err, event_id = %event_id, "failed to insert news.geosignal");
        } else {
            ingested += 1;
        }
    }

    info!(ingested = ingested, "Ingested GDELT geosignals");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_gdelt_conflict_to_gold_and_dxy() {
        let assets = map_assets("geopolitical_conflict", "Middle East");
        assert!(assets.contains(&"XAUUSD".to_string()));
        assert!(assets.contains(&"DXY".to_string()));
    }

    #[test]
    fn parses_gdelt_seendate_formats() {
        let dt = parse_seendate(Some("20260721T123000Z"));
        assert_eq!(dt.timestamp(), 1784637000);

        let dt2 = parse_seendate(Some("20260721123000"));
        assert_eq!(dt2.timestamp(), 1784637000);
    }

    #[test]
    fn categorizes_gdelt_titles() {
        let (cat, sev, sent) =
            categorize_gdelt_article("Military strike reported in regional conflict zone");
        assert_eq!(cat, "geopolitical_conflict");
        assert!(sev > 0.8);
        assert!(sent < 0.0);
    }
}
