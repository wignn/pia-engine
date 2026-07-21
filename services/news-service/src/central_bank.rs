use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::time::Duration;
use tracing::{info, warn};

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct CentralBankSource {
    pub bank: String,
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CentralBankDocument {
    pub id: String,
    pub bank: String,
    pub document_type: String,
    pub title: String,
    pub url: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub summary: Option<String>,
    pub stance: String,
    pub confidence: f64,
    pub raw_text: Option<String>,
    pub raw_json: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankStance {
    pub bank: String,
    pub stance: String,
    pub confidence: f64,
    pub latest_document_id: Option<String>,
    pub latest_document_title: Option<String>,
    pub document_count: i64,
    pub hawkish_count: i64,
    pub dovish_count: i64,
    pub neutral_count: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LatestDocumentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BankDocumentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    #[serde(alias = "type")]
    pub document_type: Option<String>,
}

const HAWKISH_PHRASES: &[&str] = &[
    "inflation pressure",
    "restrictive monetary policy",
    "restrictive policy",
    "elevated inflation",
    "upside risk to inflation",
    "above target",
    "rate hike",
    "rate hikes",
    "higher for longer",
    "policy tightening",
    "monetary tightening",
];

const DOVISH_PHRASES: &[&str] = &[
    "economic slowdown",
    "easing labor market",
    "rate cut",
    "rate cuts",
    "accommodative policy",
    "downside risk to growth",
    "below target",
    "policy easing",
    "monetary easing",
    "weaker demand",
    "disinflation",
];

pub fn classify_stance(text: &str) -> &'static str {
    let lower = text.trim().to_lowercase();
    if lower.is_empty() {
        return "unknown";
    }

    let hawkish_score = score_phrases(&lower, HAWKISH_PHRASES);
    let dovish_score = score_phrases(&lower, DOVISH_PHRASES);

    match hawkish_score.cmp(&dovish_score) {
        std::cmp::Ordering::Greater if hawkish_score - dovish_score >= 2 => "hawkish",
        std::cmp::Ordering::Less if dovish_score - hawkish_score >= 2 => "dovish",
        _ => "unknown",
    }
}

pub fn calculate_confidence(text: &str, stance: &str) -> f64 {
    let lower = text.trim().to_lowercase();
    let phrases = match stance {
        "hawkish" => HAWKISH_PHRASES,
        "dovish" => DOVISH_PHRASES,
        _ => return 0.0,
    };

    let count = score_phrases(&lower, phrases);
    (0.5 + (count as f64 * 0.1)).min(1.0)
}

fn score_phrases(text: &str, phrases: &[&str]) -> usize {
    phrases
        .iter()
        .filter(|&&phrase| text.contains(phrase))
        .count()
}

pub fn generate_doc_id(bank: &str, url_or_title: &str) -> String {
    let input = format!("{}:{}", bank.to_uppercase(), url_or_title);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

pub async fn list_latest_documents(
    State(state): State<AppState>,
    Query(query): Query<LatestDocumentsQuery>,
) -> Result<Json<Vec<CentralBankDocument>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let docs = sqlx::query_as::<_, CentralBankDocument>(
        r#"
        SELECT id, bank, document_type, title, url, published_at, summary,
               stance, confidence, raw_text, raw_json, created_at, updated_at
        FROM central_bank_documents
        ORDER BY published_at DESC NULLS LAST, created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(docs))
}

pub async fn list_bank_documents(
    State(state): State<AppState>,
    Path(bank): Path<String>,
    Query(query): Query<BankDocumentsQuery>,
) -> Result<Json<Vec<CentralBankDocument>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let docs = sqlx::query_as::<_, CentralBankDocument>(
        r#"
        SELECT id, bank, document_type, title, url, published_at, summary,
               stance, confidence, raw_text, raw_json, created_at, updated_at
        FROM central_bank_documents
        WHERE UPPER(bank) = UPPER($1)
          AND ($2::text IS NULL OR UPPER(document_type) = UPPER($2))
        ORDER BY published_at DESC NULLS LAST, created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&bank)
    .bind(query.document_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(docs))
}

pub async fn get_bank_stance(
    State(state): State<AppState>,
    Path(bank): Path<String>,
) -> Result<Json<BankStance>, (StatusCode, String)> {
    let bank_upper = bank.trim().to_uppercase();

    let latest_doc = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            f64,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        r#"
        SELECT id, title, stance, confidence, published_at
        FROM central_bank_documents
        WHERE UPPER(bank) = $1
        ORDER BY published_at DESC NULLS LAST, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&bank_upper)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r#"
        SELECT
            COUNT(*)::BIGINT as total_count,
            COUNT(*) FILTER (WHERE stance = 'hawkish')::BIGINT as hawkish_count,
            COUNT(*) FILTER (WHERE stance = 'dovish')::BIGINT as dovish_count,
            COUNT(*) FILTER (WHERE stance = 'neutral' OR stance = 'unknown')::BIGINT as neutral_count
        FROM central_bank_documents
        WHERE UPPER(bank) = $1
        "#,
    )
    .bind(&bank_upper)
    .fetch_one(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let (doc_count, hawkish_cnt, dovish_cnt, neutral_cnt) = counts;

    let stance_res = match &latest_doc {
        Some(doc) => doc.2.clone(),
        None => "neutral".to_string(),
    };

    let confidence_res = match &latest_doc {
        Some(doc) => doc.3,
        None => 0.0,
    };

    let updated_at_res = latest_doc
        .as_ref()
        .and_then(|doc| doc.4)
        .unwrap_or_else(chrono::Utc::now);

    Ok(Json(BankStance {
        bank: bank_upper,
        stance: stance_res,
        confidence: confidence_res,
        latest_document_id: latest_doc.as_ref().map(|doc| doc.0.clone()),
        latest_document_title: latest_doc.as_ref().map(|doc| doc.1.clone()),
        document_count: doc_count,
        hawkish_count: hawkish_cnt,
        dovish_count: dovish_cnt,
        neutral_count: neutral_cnt,
        updated_at: updated_at_res,
    }))
}

pub async fn run_central_bank_sync(cfg: Config, pool: sqlx::PgPool) {
    let default_sources = vec![
        (
            "FED",
            "Federal Reserve Press Releases",
            "https://www.federalreserve.gov/feeds/press_all.xml",
        ),
        (
            "ECB",
            "European Central Bank Press Releases",
            "https://www.ecb.europa.eu/rss/press.html",
        ),
        (
            "BOE",
            "Bank of England News",
            "https://www.bankofengland.co.uk/rss/news",
        ),
        (
            "BOJ",
            "Bank of Japan Press Releases",
            "https://www.boj.or.jp/en/rss/press.xml",
        ),
        (
            "RBA",
            "Reserve Bank of Australia Media Releases",
            "https://www.rba.gov.au/rss/rss-cb-media-releases.xml",
        ),
        (
            "SNB",
            "Swiss National Bank Press Releases",
            "https://www.snb.ch/en/service/rss/press",
        ),
    ];

    for (bank, name, url) in &default_sources {
        let _ = sqlx::query(
            r#"
            INSERT INTO central_bank_sources (bank, name, url, source_type, active, updated_at)
            VALUES ($1, $2, $3, 'rss', TRUE, NOW())
            ON CONFLICT (bank, url) DO NOTHING
            "#,
        )
        .bind(bank)
        .bind(name)
        .bind(url)
        .execute(&pool)
        .await;
    }

    let user_agent = cfg
        .sec_user_agent
        .as_deref()
        .unwrap_or("ATLSD-NewsService/1.0 (central-bank-monitor)");

    let client = match reqwest::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            warn!(error = %err, "failed to build HTTP client for central bank sync");
            return;
        }
    };

    info!("Starting Central Bank RSS monitor loop");

    loop {
        if let Err(err) = sync_central_banks(&client, &pool).await {
            warn!(error = %err, "Central bank monitor sync iteration failed");
        }

        tokio::time::sleep(Duration::from_secs(1800)).await;
    }
}

async fn sync_central_banks(client: &reqwest::Client, pool: &sqlx::PgPool) -> Result<(), String> {
    let sources = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT bank, name, url
        FROM central_bank_sources
        WHERE active = TRUE
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|err| format!("Failed to fetch active central bank sources: {}", err))?;

    for (bank, _name, url) in sources {
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, bank = %bank, url = %url, "failed to fetch central bank RSS feed");
                continue;
            }
        };

        if !resp.status().is_success() {
            warn!(status = %resp.status(), bank = %bank, url = %url, "non-success status fetching central bank feed");
            continue;
        }

        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(err) => {
                warn!(error = %err, bank = %bank, url = %url, "failed to read RSS feed response body");
                continue;
            }
        };

        let channel = match rss::Channel::read_from(Cursor::new(&bytes)) {
            Ok(c) => c,
            Err(err) => {
                warn!(error = %err, bank = %bank, url = %url, "failed to parse RSS channel");
                continue;
            }
        };

        for item in channel.items() {
            let title = item.title().unwrap_or("Central Bank Document").trim();
            if title.is_empty() {
                continue;
            }

            let link = item
                .link()
                .or_else(|| item.guid().map(|g| g.value()))
                .unwrap_or(&url);

            let doc_url = if link.starts_with("http") {
                link.to_string()
            } else {
                url.clone()
            };

            let summary = item
                .description()
                .or_else(|| item.content())
                .map(|s| s.trim().to_string());

            let published_at = item.pub_date().and_then(|pd| {
                chrono::DateTime::parse_from_rfc2822(pd)
                    .or_else(|_| chrono::DateTime::parse_from_rfc3339(pd))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            });

            let lower_title = title.to_lowercase();
            let doc_type = if lower_title.contains("speech") {
                "speech"
            } else if lower_title.contains("minute") {
                "minutes"
            } else if lower_title.contains("statement") {
                "statement"
            } else if lower_title.contains("report") {
                "report"
            } else {
                "press_release"
            };

            let full_text = format!("{} {}", title, summary.as_deref().unwrap_or(""));
            let stance = classify_stance(&full_text);
            let confidence = calculate_confidence(&full_text, stance);
            let doc_id = generate_doc_id(&bank, &doc_url);

            let raw_json = serde_json::json!({
                "title": title,
                "link": doc_url,
                "pubDate": item.pub_date(),
            });

            let _ = sqlx::query(
                r#"
                INSERT INTO central_bank_documents (
                    id, bank, document_type, title, url, published_at, summary,
                    stance, confidence, raw_text, raw_json, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
                ON CONFLICT (bank, url) DO UPDATE
                SET title = EXCLUDED.title,
                    published_at = COALESCE(EXCLUDED.published_at, central_bank_documents.published_at),
                    summary = COALESCE(EXCLUDED.summary, central_bank_documents.summary),
                    stance = EXCLUDED.stance,
                    confidence = EXCLUDED.confidence,
                    raw_text = EXCLUDED.raw_text,
                    raw_json = EXCLUDED.raw_json,
                    updated_at = NOW()
                "#,
            )
            .bind(&doc_id)
            .bind(&bank)
            .bind(doc_type)
            .bind(title)
            .bind(&doc_url)
            .bind(published_at)
            .bind(&summary)
            .bind(stance)
            .bind(confidence)
            .bind(&full_text)
            .bind(&raw_json)
            .execute(pool)
            .await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_hawkish_text() {
        let stance = classify_stance(
            "Inflation pressure remains elevated, requiring restrictive monetary policy.",
        );
        assert_eq!(stance, "hawkish");
    }

    #[test]
    fn classifies_dovish_text() {
        let stance = classify_stance(
            "Economic slowdown and easing labor market suggest rate cuts are appropriate.",
        );
        assert_eq!(stance, "dovish");
    }

    #[test]
    fn classifies_empty_text_as_unknown() {
        assert_eq!(classify_stance("  "), "unknown");
    }

    #[test]
    fn classifies_tied_text_as_unknown() {
        let stance =
            classify_stance("Inflation pressure remains high, but economic slowdown is clear.");
        assert_eq!(stance, "unknown");
    }

    #[test]
    fn classifies_neutral_text_as_unknown() {
        let stance = classify_stance("The committee reviewed incoming data and market conditions.");
        assert_eq!(stance, "unknown");
    }

    #[test]
    fn unknown_confidence_is_zero() {
        assert_eq!(
            calculate_confidence("rate cuts are possible", "unknown"),
            0.0
        );
    }
}
