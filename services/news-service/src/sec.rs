use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecCompany {
    pub cik: String,
    pub ticker: String,
    pub name: String,
    pub exchange: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecFiling {
    pub accession_number: String,
    pub cik: String,
    pub ticker: Option<String>,
    pub form_type: String,
    pub filing_date: chrono::NaiveDate,
    pub report_date: Option<chrono::NaiveDate>,
    pub primary_document: String,
    pub document_url: String,
    pub title: String,
    pub raw_json: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FilingsQuery {
    #[serde(alias = "symbol")]
    pub ticker: Option<String>,
    #[serde(alias = "form")]
    pub form_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn build_sec_doc_url(cik: &str, accession_number: &str, primary_document: &str) -> String {
    let cik_trimmed = cik.trim_start_matches('0');
    let cik_numeric = if cik_trimmed.is_empty() {
        "0"
    } else {
        cik_trimmed
    };
    let accession_no_dashes = accession_number.replace('-', "");
    format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
        cik_numeric, accession_no_dashes, primary_document
    )
}

pub fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_uppercase()
}

fn rotating_company_batch<T>(items: &[T], batch_size: usize, timestamp_secs: i64) -> Vec<&T> {
    if items.is_empty() || batch_size == 0 {
        return Vec::new();
    }

    let batch_size = batch_size.min(items.len());
    let hour = timestamp_secs.max(0) as usize / 3600;
    let start = hour.saturating_mul(batch_size) % items.len();

    (0..batch_size)
        .map(|idx| &items[(start + idx) % items.len()])
        .collect()
}

pub async fn list_filings(
    State(state): State<AppState>,
    Query(query): Query<FilingsQuery>,
) -> Result<Json<Vec<SecFiling>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let filings = sqlx::query_as::<_, SecFiling>(
        r#"
        SELECT accession_number, cik, ticker, form_type, filing_date, report_date,
               primary_document, document_url, title, raw_json, created_at, updated_at
        FROM sec_filings
        WHERE ($1::text IS NULL OR UPPER(ticker) = UPPER($1))
          AND ($2::text IS NULL OR UPPER(form_type) = UPPER($2))
        ORDER BY filing_date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(query.ticker)
    .bind(query.form_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(filings))
}

pub async fn get_filing(
    State(state): State<AppState>,
    Path(accession_number): Path<String>,
) -> Result<Json<SecFiling>, (StatusCode, String)> {
    let filing = sqlx::query_as::<_, SecFiling>(
        r#"
        SELECT accession_number, cik, ticker, form_type, filing_date, report_date,
               primary_document, document_url, title, raw_json, created_at, updated_at
        FROM sec_filings
        WHERE accession_number = $1
        "#,
    )
    .bind(&accession_number)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    match filing {
        Some(f) => Ok(Json(f)),
        None => Err((StatusCode::NOT_FOUND, "Filing not found".to_string())),
    }
}

pub async fn get_company(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<SecCompany>, (StatusCode, String)> {
    let normalized = normalize_symbol(&symbol);
    let company = sqlx::query_as::<_, SecCompany>(
        r#"
        SELECT cik, ticker, name, exchange, updated_at
        FROM sec_companies
        WHERE UPPER(ticker) = $1
        "#,
    )
    .bind(&normalized)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    match company {
        Some(c) => Ok(Json(c)),
        None => Err((StatusCode::NOT_FOUND, "Company not found".to_string())),
    }
}

pub async fn run_sec_sync(cfg: Config, pool: sqlx::PgPool) {
    let user_agent = match cfg
        .sec_user_agent
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        Some(ua) => ua.to_string(),
        None => {
            warn!("SEC_USER_AGENT not configured; SEC EDGAR background sync is disabled");
            return;
        }
    };

    let client = match reqwest::Client::builder()
        .user_agent(&user_agent)
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            warn!(error = %err, "failed to build HTTP client for SEC sync");
            return;
        }
    };

    info!("Starting SEC EDGAR sync loop");

    loop {
        if let Err(err) = sync_sec_data(&client, &pool).await {
            warn!(error = %err, "SEC EDGAR sync iteration failed");
        }

        tokio::time::sleep(Duration::from_secs(cfg.sec_poll_sec)).await;
    }
}

async fn sync_sec_data(client: &reqwest::Client, pool: &sqlx::PgPool) -> Result<(), String> {
    // 1. Fetch company_tickers.json
    let tickers_url = "https://www.sec.gov/files/company_tickers.json";
    let resp = client
        .get(tickers_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch company tickers: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Company tickers API returned status {}",
            resp.status()
        ));
    }

    let tickers_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse company tickers JSON: {}", e))?;

    // company_tickers.json is an object with numerical keys ("0", "1", ...)
    let entries = match tickers_json.as_object() {
        Some(map) => map,
        None => return Err("company_tickers.json is not a JSON object".to_string()),
    };

    let mut fetched_companies: Vec<(String, String, String)> = Vec::new();

    for (_key, val) in entries {
        let cik_raw = match val.get("cik_str") {
            Some(v) => match v.as_u64() {
                Some(num) => format!("{:010}", num),
                None => match v.as_str() {
                    Some(s) => format!("{:0>10}", s),
                    None => continue,
                },
            },
            None => continue,
        };

        let ticker = match val.get("ticker").and_then(|v| v.as_str()) {
            Some(t) => t.trim().to_uppercase(),
            None => continue,
        };

        let title = match val.get("title").and_then(|v| v.as_str()) {
            Some(t) => t.trim().to_string(),
            None => continue,
        };

        if ticker.is_empty() || title.is_empty() {
            continue;
        }

        // Upsert company into sec_companies
        let _ = sqlx::query(
            r#"
            WITH removed AS (
                DELETE FROM sec_companies
                WHERE (cik = $1 OR ticker = $2)
                  AND NOT (cik = $1 AND ticker = $2)
            )
            INSERT INTO sec_companies (cik, ticker, name, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (cik) DO UPDATE
            SET ticker = EXCLUDED.ticker,
                name = EXCLUDED.name,
                updated_at = NOW()
            "#,
        )
        .bind(&cik_raw)
        .bind(&ticker)
        .bind(&title)
        .execute(pool)
        .await;

        fetched_companies.push((cik_raw, ticker, title));
    }

    fetched_companies.sort_by(|a, b| a.1.cmp(&b.1));

    info!(
        count = fetched_companies.len(),
        "Upserted SEC company tickers"
    );

    let filing_batch =
        rotating_company_batch(&fetched_companies, 100, chrono::Utc::now().timestamp());

    for (cik, ticker, title) in filing_batch {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let filings_url = format!("https://data.sec.gov/submissions/CIK{}.json", cik);
        let resp = match client.get(&filings_url).send().await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, cik = %cik, "failed to fetch SEC submissions");
                continue;
            }
        };

        if !resp.status().is_success() {
            continue;
        }

        let sub_json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(_) => continue,
        };

        let recent = match sub_json.pointer("/filings/recent") {
            Some(r) => r,
            None => continue,
        };

        let accessions = match recent.get("accessionNumber").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };

        let forms = match recent.get("form").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };

        let filing_dates = match recent.get("filingDate").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };

        let report_dates = recent.get("reportDate").and_then(|v| v.as_array());
        let primary_docs = match recent.get("primaryDocument").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };
        let primary_doc_descs = recent
            .get("primaryDocDescription")
            .and_then(|v| v.as_array());

        let len = accessions.len();
        for i in 0..len {
            let acc_num = match accessions.get(i).and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let form = match forms.get(i).and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let filing_date_str = match filing_dates.get(i).and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let filing_date = match chrono::NaiveDate::parse_from_str(filing_date_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            };
            let report_date = report_dates
                .and_then(|arr| arr.get(i))
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

            let primary_doc = match primary_docs.get(i).and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };

            let doc_desc = primary_doc_descs
                .and_then(|arr| arr.get(i))
                .and_then(|v| v.as_str())
                .unwrap_or(form);

            let doc_url = build_sec_doc_url(cik, acc_num, primary_doc);
            let filing_title = format!("{} ({}) - {}", ticker, form, doc_desc);

            let raw_obj = serde_json::json!({
                "accessionNumber": acc_num,
                "form": form,
                "filingDate": filing_date_str,
                "primaryDocument": primary_doc,
                "companyName": title,
            });

            let _ = sqlx::query(
                r#"
                INSERT INTO sec_filings (
                    accession_number, cik, ticker, form_type, filing_date, report_date,
                    primary_document, document_url, title, raw_json, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
                ON CONFLICT (accession_number) DO UPDATE
                SET ticker = EXCLUDED.ticker,
                    form_type = EXCLUDED.form_type,
                    filing_date = EXCLUDED.filing_date,
                    report_date = EXCLUDED.report_date,
                    primary_document = EXCLUDED.primary_document,
                    document_url = EXCLUDED.document_url,
                    title = EXCLUDED.title,
                    raw_json = EXCLUDED.raw_json,
                    updated_at = NOW()
                "#,
            )
            .bind(acc_num)
            .bind(cik)
            .bind(ticker)
            .bind(form)
            .bind(filing_date)
            .bind(report_date)
            .bind(primary_doc)
            .bind(&doc_url)
            .bind(&filing_title)
            .bind(&raw_obj)
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
    fn builds_sec_document_url_correctly() {
        let url = build_sec_doc_url("0000320193", "0000320193-23-000106", "aapl-20230930.htm");
        assert_eq!(
            url,
            "https://www.sec.gov/Archives/edgar/data/320193/000032019323000106/aapl-20230930.htm"
        );
    }

    #[test]
    fn normalizes_symbol_correctly() {
        assert_eq!(normalize_symbol(" aapl "), "AAPL");
        assert_eq!(normalize_symbol("msft"), "MSFT");
    }

    #[test]
    fn deserializes_filings_query_aliases() {
        let query: FilingsQuery = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "form": "10-K",
            "limit": 25,
        }))
        .unwrap();

        assert_eq!(query.ticker.as_deref(), Some("AAPL"));
        assert_eq!(query.form_type.as_deref(), Some("10-K"));
        assert_eq!(query.limit, Some(25));
    }

    #[test]
    fn rotates_company_batches_by_hour() {
        let items = vec!["A", "B", "C", "D", "E"];
        assert_eq!(rotating_company_batch(&items, 2, 0), vec![&"A", &"B"]);
        assert_eq!(rotating_company_batch(&items, 2, 3600), vec![&"C", &"D"]);
        assert_eq!(rotating_company_batch(&items, 2, 7200), vec![&"E", &"A"]);
    }

    #[test]
    fn handles_cik_without_leading_zeros() {
        let url = build_sec_doc_url("320193", "0000320193-23-000106", "aapl-20230930.htm");
        assert_eq!(
            url,
            "https://www.sec.gov/Archives/edgar/data/320193/000032019323000106/aapl-20230930.htm"
        );
    }
}
