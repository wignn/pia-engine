use atlsd_eventbus::{subjects, EventPublisher};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::config::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptionContractData {
    pub contract_symbol: String,
    pub symbol: String,
    pub option_type: String, // "call" or "put"
    pub strike: f64,
    pub expiration_date: String, // "YYYY-MM-DD"
    pub mark_price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub gex: f64,
    pub open_interest: u64,
    pub volume: u64,
}

// Deribit JSON responses
#[derive(Debug, serde::Deserialize)]
struct DeribitResponse {
    result: Option<Vec<DeribitBookSummary>>,
}

#[derive(Debug, serde::Deserialize)]
struct DeribitBookSummary {
    instrument_name: String,
    underlying_price: Option<f64>,
    mark_price: Option<f64>,
    bid_price: Option<f64>,
    ask_price: Option<f64>,
    mark_iv: Option<f64>,
    open_interest: Option<f64>,
    volume: Option<f64>,
}

fn parse_deribit_exp(exp_str: &str) -> Option<(String, f64)> {
    let months = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    for (idx, &m) in months.iter().enumerate() {
        if let Some(pos) = exp_str.find(m) {
            let day: u32 = exp_str[..pos].parse().ok()?;
            let year_short: u32 = exp_str[pos + 3..].parse().ok()?;
            let year = 2000 + year_short;
            let month = (idx + 1) as u32;
            let date_str = format!("{:04}-{:02}-{:02}", year, month, day);

            let naive_date = chrono::NaiveDate::from_ymd_opt(year as i32, month, day)?;
            let naive_datetime = naive_date.and_hms_opt(8, 0, 0)?;
            let timestamp = naive_datetime.and_utc().timestamp();
            let now = Utc::now().timestamp();
            let time_years = ((timestamp - now) as f64) / (365.25 * 86400.0);

            return Some((date_str, time_years.max(0.001)));
        }
    }
    None
}

pub async fn fetch_deribit_options(
    client: &reqwest::Client,
    currency: &str,
) -> anyhow::Result<(f64, Vec<OptionContractData>)> {
    let url = format!(
        "https://www.deribit.com/api/v2/public/get_book_summary_by_currency?currency={}&kind=option",
        currency
    );
    let res: DeribitResponse = client.get(&url).send().await?.json().await?;
    let summaries = res.result.unwrap_or_default();

    let underlying_price = summaries
        .iter()
        .find_map(|s| s.underlying_price)
        .unwrap_or(0.0);

    let mut contracts = Vec::new();

    for item in summaries {
        let parts: Vec<&str> = item.instrument_name.split('-').collect();
        if parts.len() < 4 {
            continue;
        }

        let sym = parts[0].to_uppercase();
        let exp_str = parts[1];
        let strike: f64 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let opt_flag = parts[3];
        let option_type = if opt_flag.eq_ignore_ascii_case("C") {
            "call"
        } else if opt_flag.eq_ignore_ascii_case("P") {
            "put"
        } else {
            continue;
        };

        let expiration_date = parse_deribit_exp(exp_str)
            .map(|(date, _)| date)
            .unwrap_or_else(|| format!("{}-exp", exp_str));

        let raw_mark = item.mark_price.unwrap_or(0.0);
        let mark_price = if raw_mark > 0.0 && raw_mark < 10.0 && underlying_price > 0.0 {
            raw_mark * underlying_price
        } else {
            raw_mark
        };

        let raw_bid = item.bid_price;
        let bid = raw_bid.map(|p| {
            if p > 0.0 && p < 10.0 && underlying_price > 0.0 {
                p * underlying_price
            } else {
                p
            }
        });

        let raw_ask = item.ask_price;
        let ask = raw_ask.map(|p| {
            if p > 0.0 && p < 10.0 && underlying_price > 0.0 {
                p * underlying_price
            } else {
                p
            }
        });

        let iv = item
            .mark_iv
            .map(|v| if v > 1.0 { v / 100.0 } else { v })
            .unwrap_or(0.5);

        let open_interest = item.open_interest.unwrap_or(0.0).max(0.0) as u64;
        let volume = item.volume.unwrap_or(0.0).max(0.0) as u64;

        contracts.push(OptionContractData {
            contract_symbol: item.instrument_name,
            symbol: sym,
            option_type: option_type.to_string(),
            strike,
            expiration_date,
            mark_price,
            bid,
            ask,
            implied_volatility: iv,
            delta: 0.0,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            gex: 0.0,
            open_interest,
            volume,
        });
    }

    Ok((underlying_price, contracts))
}

async fn publish_options_data(
    broker: &dyn EventPublisher,
    symbol: &str,
    underlying_price: f64,
    contracts: &[OptionContractData],
) {
    let chain_payload = json!({
        "symbol": symbol,
        "underlying_price": underlying_price,
        "contracts": contracts,
        "updated_at": Utc::now().to_rfc3339(),
    });

    if let Err(e) = broker
        .publish_str(
            subjects::MARKET_OPTIONS_CHAIN_V1,
            &chain_payload.to_string(),
        )
        .await
    {
        error!(symbol = symbol, error = %e, "failed to publish market.options.chain");
    }
}

pub async fn run_options_feed(cfg: Arc<Config>, broker: Arc<dyn EventPublisher>) {
    info!(
        worker = "options_feed",
        interval_sec = cfg.options_sync_sec,
        "starting options feed worker"
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default();

    loop {
        for currency in &["BTC", "ETH"] {
            match fetch_deribit_options(&client, currency).await {
                Ok((underlying_price, contracts)) => {
                    if !contracts.is_empty() {
                        publish_options_data(&*broker, currency, underlying_price, &contracts)
                            .await;
                    }
                }
                Err(e) => {
                    error!(worker = "options_feed", symbol = currency, error = %e, "failed to fetch Deribit options");
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(cfg.options_sync_sec)).await;
    }
}

#[allow(dead_code)]
pub async fn run(cfg: Arc<Config>, broker: Arc<dyn EventPublisher>) {
    run_options_feed(cfg, broker).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deribit_exp() {
        let res = parse_deribit_exp("26JUL26");
        assert!(res.is_some());
        let (date_str, _time_years) = res.unwrap();
        assert_eq!(date_str, "2026-07-26");
    }
}
