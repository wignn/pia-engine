use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CotReportRow {
    pub market_code: String,
    pub market_name: String,
    pub report_date: NaiveDate,
    pub report_type: String,
    pub commercial_long: Option<i64>,
    pub commercial_short: Option<i64>,
    pub noncommercial_long: Option<i64>,
    pub noncommercial_short: Option<i64>,
    pub nonreportable_long: Option<i64>,
    pub nonreportable_short: Option<i64>,
    pub open_interest: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedCotReport {
    pub market_code: String,
    pub market_name: String,
    pub report_date: NaiveDate,
    pub report_type: String,
    pub commercial_long: Option<i64>,
    pub commercial_short: Option<i64>,
    pub commercial_net: Option<i64>,
    pub commercial_net_wow: Option<i64>,
    pub noncommercial_long: Option<i64>,
    pub noncommercial_short: Option<i64>,
    pub noncommercial_net: Option<i64>,
    pub noncommercial_net_wow: Option<i64>,
    pub nonreportable_long: Option<i64>,
    pub nonreportable_short: Option<i64>,
    pub open_interest: Option<i64>,
    pub open_interest_wow: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CotMarketSummary {
    pub market_code: String,
    pub symbol: Option<String>,
    pub asset_class: Option<String>,
    pub display_name: String,
    pub latest_report_date: Option<NaiveDate>,
    pub commercial_net: Option<i64>,
    pub noncommercial_net: Option<i64>,
    pub open_interest: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CotQuery {
    pub report_type: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

pub async fn list_cot_markets(State(state): State<AppState>) -> Json<serde_json::Value> {
    let result = query_cot_markets(&state.db).await;
    match result {
        Ok(markets) => Json(serde_json::json!({ "data": markets })),
        Err(err) => {
            error!(error = %err, "failed to query cot markets");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_cot_by_market(
    State(state): State<AppState>,
    Path(market_code): Path<String>,
    Query(params): Query<CotQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let report_type = params
        .report_type
        .as_deref()
        .unwrap_or("legacy_futures_only");

    let result = query_cot_reports(
        &state.db,
        &market_code,
        report_type,
        params.from,
        params.to,
        limit,
    )
    .await;

    match result {
        Ok(reports) => Json(serde_json::json!({
            "market_code": market_code,
            "report_type": report_type,
            "data": reports,
        })),
        Err(err) => {
            error!(error = %err, market_code = %market_code, "failed to query cot reports by market");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_cot_by_symbol(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<CotQuery>,
) -> Json<serde_json::Value> {
    let market_code = match resolve_symbol_to_market_code(&state.db, &symbol).await {
        Ok(Some(code)) => code,
        Ok(None) => {
            return Json(serde_json::json!({ "error": format!("symbol '{}' not found", symbol) }));
        }
        Err(err) => {
            error!(error = %err, symbol = %symbol, "failed to resolve cot symbol");
            return Json(serde_json::json!({ "error": "internal server error" }));
        }
    };

    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let report_type = params
        .report_type
        .as_deref()
        .unwrap_or("legacy_futures_only");

    let result = query_cot_reports(
        &state.db,
        &market_code,
        report_type,
        params.from,
        params.to,
        limit,
    )
    .await;

    match result {
        Ok(reports) => Json(serde_json::json!({
            "symbol": symbol,
            "market_code": market_code,
            "report_type": report_type,
            "data": reports,
        })),
        Err(err) => {
            error!(error = %err, symbol = %symbol, market_code = %market_code, "failed to query cot reports by symbol");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

async fn resolve_symbol_to_market_code(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, String>(
        r#"
        SELECT market_code
        FROM cot_market_map
        WHERE UPPER(symbol) = UPPER($1) OR UPPER(market_code) = UPPER($1)
        LIMIT 1
        "#,
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

async fn query_cot_markets(pool: &PgPool) -> Result<Vec<CotMarketSummary>, sqlx::Error> {
    sqlx::query_as::<_, CotMarketSummary>(
        r#"
        SELECT
            m.market_code,
            m.symbol,
            m.asset_class,
            COALESCE(m.display_name, r.market_name, m.market_code) AS display_name,
            r.report_date AS latest_report_date,
            (r.commercial_long - r.commercial_short) AS commercial_net,
            (r.noncommercial_long - r.noncommercial_short) AS noncommercial_net,
            r.open_interest
        FROM cot_market_map m
        LEFT JOIN LATERAL (
            SELECT market_name, report_date, commercial_long, commercial_short, noncommercial_long, noncommercial_short, open_interest
            FROM cot_reports
            WHERE market_code = m.market_code
            ORDER BY report_date DESC
            LIMIT 1
        ) r ON true
        ORDER BY m.asset_class, m.symbol
        "#,
    )
    .fetch_all(pool)
    .await
}

async fn query_cot_reports(
    pool: &PgPool,
    market_code: &str,
    report_type: &str,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<DerivedCotReport>, sqlx::Error> {
    // Fetch limit + 1 to calculate WoW change for the last report in the page
    let raw_reports = sqlx::query_as::<_, CotReportRow>(
        r#"
        SELECT market_code, market_name, report_date, report_type,
               commercial_long, commercial_short,
               noncommercial_long, noncommercial_short,
               nonreportable_long, nonreportable_short,
               open_interest, created_at, updated_at
        FROM cot_reports
        WHERE market_code = $1
          AND report_type = $2
          AND ($3::date IS NULL OR report_date >= $3)
          AND ($4::date IS NULL OR report_date <= $4)
        ORDER BY report_date DESC
        LIMIT $5
        "#,
    )
    .bind(market_code)
    .bind(report_type)
    .bind(from)
    .bind(to)
    .bind(limit + 1)
    .fetch_all(pool)
    .await?;

    let mut derived = compute_derived_reports(raw_reports);
    if derived.len() > limit as usize {
        derived.truncate(limit as usize);
    }
    Ok(derived)
}

pub fn compute_derived_reports(reports: Vec<CotReportRow>) -> Vec<DerivedCotReport> {
    let mut derived = Vec::with_capacity(reports.len());

    for i in 0..reports.len() {
        let curr = &reports[i];

        let comm_net = match (curr.commercial_long, curr.commercial_short) {
            (Some(l), Some(s)) => Some(l - s),
            _ => None,
        };

        let noncomm_net = match (curr.noncommercial_long, curr.noncommercial_short) {
            (Some(l), Some(s)) => Some(l - s),
            _ => None,
        };

        let (comm_net_wow, noncomm_net_wow, open_interest_wow) =
            if let Some(prev) = reports.get(i + 1) {
                let prev_comm_net = match (prev.commercial_long, prev.commercial_short) {
                    (Some(l), Some(s)) => Some(l - s),
                    _ => None,
                };
                let c_wow = match (comm_net, prev_comm_net) {
                    (Some(c), Some(p)) => Some(c - p),
                    _ => None,
                };

                let prev_noncomm_net = match (prev.noncommercial_long, prev.noncommercial_short) {
                    (Some(l), Some(s)) => Some(l - s),
                    _ => None,
                };
                let nc_wow = match (noncomm_net, prev_noncomm_net) {
                    (Some(c), Some(p)) => Some(c - p),
                    _ => None,
                };

                let oi_wow = match (curr.open_interest, prev.open_interest) {
                    (Some(c), Some(p)) => Some(c - p),
                    _ => None,
                };

                (c_wow, nc_wow, oi_wow)
            } else {
                (None, None, None)
            };

        derived.push(DerivedCotReport {
            market_code: curr.market_code.clone(),
            market_name: curr.market_name.clone(),
            report_date: curr.report_date,
            report_type: curr.report_type.clone(),
            commercial_long: curr.commercial_long,
            commercial_short: curr.commercial_short,
            commercial_net: comm_net,
            commercial_net_wow: comm_net_wow,
            noncommercial_long: curr.noncommercial_long,
            noncommercial_short: curr.noncommercial_short,
            noncommercial_net: noncomm_net,
            noncommercial_net_wow: noncomm_net_wow,
            nonreportable_long: curr.nonreportable_long,
            nonreportable_short: curr.nonreportable_short,
            open_interest: curr.open_interest,
            open_interest_wow,
            updated_at: curr.updated_at,
        });
    }

    derived
}

// --- Background Sync for CFTC COT Positioning ---

struct SeedMarket {
    market_code: &'static str,
    symbol: &'static str,
    asset_class: &'static str,
    display_name: &'static str,
}

const SEED_MARKET_MAP: &[SeedMarket] = &[
    SeedMarket {
        market_code: "096742",
        symbol: "EUR",
        asset_class: "FX",
        display_name: "Euro FX",
    },
    SeedMarket {
        market_code: "090741",
        symbol: "CAD",
        asset_class: "FX",
        display_name: "Canadian Dollar",
    },
    SeedMarket {
        market_code: "099741",
        symbol: "GBP",
        asset_class: "FX",
        display_name: "British Pound",
    },
    SeedMarket {
        market_code: "097741",
        symbol: "JPY",
        asset_class: "FX",
        display_name: "Japanese Yen",
    },
    SeedMarket {
        market_code: "092741",
        symbol: "CHF",
        asset_class: "FX",
        display_name: "Swiss Franc",
    },
    SeedMarket {
        market_code: "023741",
        symbol: "AUD",
        asset_class: "FX",
        display_name: "Australian Dollar",
    },
    SeedMarket {
        market_code: "112741",
        symbol: "NZD",
        asset_class: "FX",
        display_name: "New Zealand Dollar",
    },
    SeedMarket {
        market_code: "088691",
        symbol: "XAUUSD",
        asset_class: "Commodities",
        display_name: "Gold",
    },
    SeedMarket {
        market_code: "084691",
        symbol: "XAGUSD",
        asset_class: "Commodities",
        display_name: "Silver",
    },
    SeedMarket {
        market_code: "067651",
        symbol: "WTI",
        asset_class: "Commodities",
        display_name: "Crude Oil Light Sweet",
    },
    SeedMarket {
        market_code: "13874A",
        symbol: "SPX",
        asset_class: "Indices",
        display_name: "E-Mini S&P 500",
    },
    SeedMarket {
        market_code: "209742",
        symbol: "NDX",
        asset_class: "Indices",
        display_name: "E-Mini Nasdaq 100",
    },
    SeedMarket {
        market_code: "133741",
        symbol: "BTC",
        asset_class: "Crypto",
        display_name: "Bitcoin",
    },
];

#[derive(Debug, PartialEq, Eq)]
pub struct CotParsedRecord {
    pub market_name: String,
    pub report_date: NaiveDate,
    pub market_code: String,
    pub open_interest: Option<i64>,
    pub noncommercial_long: Option<i64>,
    pub noncommercial_short: Option<i64>,
    pub commercial_long: Option<i64>,
    pub commercial_short: Option<i64>,
    pub nonreportable_long: Option<i64>,
    pub nonreportable_short: Option<i64>,
}

pub fn parse_cot_line(line: &str) -> Option<CotParsedRecord> {
    let fields: Vec<&str> = line
        .split(',')
        .map(|s| s.trim().trim_matches('"'))
        .collect();

    if fields.len() < 9 {
        return None;
    }

    let market_name = fields[0].to_string();
    if market_name.is_empty() || market_name.eq_ignore_ascii_case("Market_and_Exchange_Names") {
        return None;
    }

    let report_date = parse_cot_date(fields[1])?;
    let market_code = fields[2].to_string();
    if market_code.is_empty() {
        return None;
    }

    let open_interest = fields[3].parse::<i64>().ok();
    let noncommercial_long = fields[4].parse::<i64>().ok();
    let noncommercial_short = fields[5].parse::<i64>().ok();
    let commercial_long = fields[7].parse::<i64>().ok();
    let commercial_short = fields[8].parse::<i64>().ok();
    let nonreportable_long = fields.get(11).and_then(|s| s.parse::<i64>().ok());
    let nonreportable_short = fields.get(12).and_then(|s| s.parse::<i64>().ok());

    Some(CotParsedRecord {
        market_name,
        report_date,
        market_code,
        open_interest,
        noncommercial_long,
        noncommercial_short,
        commercial_long,
        commercial_short,
        nonreportable_long,
        nonreportable_short,
    })
}

fn parse_cot_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%y%m%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y%m%d"))
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .ok()
}

pub async fn run_cot_sync(config: Config, pool: PgPool) {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    info!(
        refresh_sec = config.cot_sync_sec,
        url = %config.cot_data_url,
        "CFTC COT positioning sync started"
    );

    loop {
        if let Err(err) = seed_market_map(&pool).await {
            warn!(error = %err, "failed to seed cot_market_map");
        }

        if let Err(err) = fetch_and_sync_cot(&config, &pool, &http).await {
            warn!(error = %err, "CFTC COT data sync failed");
        }

        tokio::time::sleep(Duration::from_secs(config.cot_sync_sec)).await;
    }
}

async fn seed_market_map(pool: &PgPool) -> Result<(), sqlx::Error> {
    for seed in SEED_MARKET_MAP {
        sqlx::query(
            r#"
            INSERT INTO cot_market_map (market_code, symbol, asset_class, display_name)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (market_code) DO UPDATE SET
                symbol = EXCLUDED.symbol,
                asset_class = EXCLUDED.asset_class,
                display_name = EXCLUDED.display_name
            "#,
        )
        .bind(seed.market_code)
        .bind(seed.symbol)
        .bind(seed.asset_class)
        .bind(seed.display_name)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn fetch_and_sync_cot(
    config: &Config,
    pool: &PgPool,
    http: &reqwest::Client,
) -> anyhow::Result<usize> {
    info!(url = %config.cot_data_url, "fetching CFTC COT public report");

    let resp = http.get(&config.cot_data_url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("CFTC server returned status {}", resp.status());
    }

    let text = resp.text().await?;
    let mut count = 0usize;

    for line in text.lines() {
        let record = match parse_cot_line(line) {
            Some(rec) => rec,
            None => continue,
        };

        // Ensure cot_market_map entry exists
        sqlx::query(
            r#"
            INSERT INTO cot_market_map (market_code, symbol, asset_class, display_name)
            VALUES ($1, $1, 'Other', $2)
            ON CONFLICT (market_code) DO NOTHING
            "#,
        )
        .bind(&record.market_code)
        .bind(&record.market_name)
        .execute(pool)
        .await?;

        // Upsert into cot_reports
        sqlx::query(
            r#"
            INSERT INTO cot_reports (
                market_code, market_name, report_date, report_type,
                commercial_long, commercial_short,
                noncommercial_long, noncommercial_short,
                nonreportable_long, nonreportable_short,
                open_interest, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'legacy_futures_only', $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            ON CONFLICT (report_type, market_code, report_date) DO UPDATE SET
                market_name = EXCLUDED.market_name,
                commercial_long = EXCLUDED.commercial_long,
                commercial_short = EXCLUDED.commercial_short,
                noncommercial_long = EXCLUDED.noncommercial_long,
                noncommercial_short = EXCLUDED.noncommercial_short,
                nonreportable_long = EXCLUDED.nonreportable_long,
                nonreportable_short = EXCLUDED.nonreportable_short,
                open_interest = EXCLUDED.open_interest,
                updated_at = NOW()
            "#,
        )
        .bind(&record.market_code)
        .bind(&record.market_name)
        .bind(record.report_date)
        .bind(record.commercial_long)
        .bind(record.commercial_short)
        .bind(record.noncommercial_long)
        .bind(record.noncommercial_short)
        .bind(record.nonreportable_long)
        .bind(record.nonreportable_short)
        .bind(record.open_interest)
        .execute(pool)
        .await?;

        count += 1;
    }

    info!(count, "CFTC COT data sync complete");
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_net_positions_and_wow_changes() {
        let now = Utc::now();
        let date2 = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        let date1 = NaiveDate::from_ymd_opt(2026, 7, 7).unwrap();

        let row2 = CotReportRow {
            market_code: "096742".to_string(),
            market_name: "EURO FX".to_string(),
            report_date: date2,
            report_type: "legacy_futures_only".to_string(),
            commercial_long: Some(150000),
            commercial_short: Some(200000),
            noncommercial_long: Some(220000),
            noncommercial_short: Some(180000),
            nonreportable_long: Some(30000),
            nonreportable_short: Some(20000),
            open_interest: Some(600000),
            created_at: now,
            updated_at: now,
        };

        let row1 = CotReportRow {
            market_code: "096742".to_string(),
            market_name: "EURO FX".to_string(),
            report_date: date1,
            report_type: "legacy_futures_only".to_string(),
            commercial_long: Some(120000),
            commercial_short: Some(210000),
            noncommercial_long: Some(200000),
            noncommercial_short: Some(190000),
            nonreportable_long: Some(28000),
            nonreportable_short: Some(22000),
            open_interest: Some(580000),
            created_at: now,
            updated_at: now,
        };

        let derived = compute_derived_reports(vec![row2, row1]);

        assert_eq!(derived.len(), 2);

        // Week 2 (latest)
        assert_eq!(derived[0].commercial_net, Some(-50000));
        assert_eq!(derived[0].noncommercial_net, Some(40000));
        assert_eq!(derived[0].commercial_net_wow, Some(40000)); // (-50000) - (-90000) = +40000
        assert_eq!(derived[0].noncommercial_net_wow, Some(30000)); // 40000 - 10000 = +30000
        assert_eq!(derived[0].open_interest_wow, Some(20000));

        // Week 1 (oldest)
        assert_eq!(derived[1].commercial_net, Some(-90000));
        assert_eq!(derived[1].noncommercial_net, Some(10000));
        assert_eq!(derived[1].commercial_net_wow, None);
        assert_eq!(derived[1].noncommercial_net_wow, None);
        assert_eq!(derived[1].open_interest_wow, None);
    }

    #[test]
    fn parses_valid_cftc_cot_line() {
        let line = "\"CANADIAN DOLLAR - CHICAGO MERCANTILE EXCHANGE\",\"260714\",\"090741\",\"185432\",\"45210\",\"62310\",\"1200\",\"110000\",\"95000\",\"156410\",\"158510\",\"29022\",\"26922\"";
        let parsed = parse_cot_line(line).expect("should parse valid line");

        assert_eq!(
            parsed.market_name,
            "CANADIAN DOLLAR - CHICAGO MERCANTILE EXCHANGE"
        );
        assert_eq!(
            parsed.report_date,
            NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()
        );
        assert_eq!(parsed.market_code, "090741");
        assert_eq!(parsed.open_interest, Some(185432));
        assert_eq!(parsed.noncommercial_long, Some(45210));
        assert_eq!(parsed.noncommercial_short, Some(62310));
        assert_eq!(parsed.commercial_long, Some(110000));
        assert_eq!(parsed.commercial_short, Some(95000));
        assert_eq!(parsed.nonreportable_long, Some(29022));
        assert_eq!(parsed.nonreportable_short, Some(26922));
    }

    #[test]
    fn skips_invalid_cot_line_defensively() {
        assert!(parse_cot_line("").is_none());
        assert!(parse_cot_line("Market_and_Exchange_Names,As_of_Date,CFTC_Code").is_none());
        assert!(parse_cot_line("CANADIAN DOLLAR,invalid_date,090741,10,20,30,40,50,60").is_none());
    }
}
