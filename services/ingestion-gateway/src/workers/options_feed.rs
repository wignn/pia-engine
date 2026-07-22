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

/// Standard normal cumulative distribution function N(x)
fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal probability density function N'(x)
fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Error function erf(x) using Abramowitz and Stegun 7.1.26 approximation
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs();
    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    sign * y
}

/// Calculates Black-Scholes Greeks (delta, gamma, theta, vega).
pub fn calculate_greeks(
    option_type: &str,
    strike: f64,
    underlying_price: f64,
    time_years: f64,
    iv: f64,
    risk_free_rate: f64,
) -> (f64, f64, f64, f64) {
    if time_years <= 0.0 || iv <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 {
        let delta = match option_type.to_lowercase().as_str() {
            "call" if underlying_price > strike => 1.0,
            "put" if underlying_price < strike => -1.0,
            _ => 0.0,
        };
        return (delta, 0.0, 0.0, 0.0);
    }

    let sqrt_t = time_years.sqrt();
    let d1 = ((underlying_price / strike).ln() + (risk_free_rate + 0.5 * iv * iv) * time_years)
        / (iv * sqrt_t);
    let d2 = d1 - iv * sqrt_t;

    let n_d1 = norm_cdf(d1);
    let n_d2 = norm_cdf(d2);
    let n_prime_d1 = norm_pdf(d1);

    let (delta, theta) = match option_type.to_lowercase().as_str() {
        "call" => {
            let delta = n_d1;
            let theta = -(underlying_price * n_prime_d1 * iv) / (2.0 * sqrt_t)
                - risk_free_rate * strike * (-risk_free_rate * time_years).exp() * n_d2;
            (delta, theta)
        }
        "put" => {
            let delta = n_d1 - 1.0;
            let theta = -(underlying_price * n_prime_d1 * iv) / (2.0 * sqrt_t)
                + risk_free_rate * strike * (-risk_free_rate * time_years).exp() * norm_cdf(-d2);
            (delta, theta)
        }
        _ => (0.0, 0.0),
    };

    let gamma = n_prime_d1 / (underlying_price * iv * sqrt_t);
    let vega = underlying_price * n_prime_d1 * sqrt_t;

    (delta, gamma, theta, vega)
}

/// Calculates Dollar Gamma Exposure (GEX).
pub fn calculate_gex(gamma: f64, underlying_price: f64, open_interest: f64, is_call: bool) -> f64 {
    let sign = if is_call { 1.0 } else { -1.0 };
    sign * gamma * underlying_price * 100.0 * open_interest * underlying_price
}

/// Calculates Max Pain strike for a set of contracts.
pub fn calculate_max_pain(contracts: &[OptionContractData]) -> f64 {
    if contracts.is_empty() {
        return 0.0;
    }

    let mut strikes: Vec<f64> = contracts.iter().map(|c| c.strike).collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    strikes.dedup();

    if strikes.is_empty() {
        return 0.0;
    }

    let mut min_payout = f64::MAX;
    let mut max_pain_strike = strikes[0];

    for &cand_strike in &strikes {
        let mut total_payout = 0.0;
        for c in contracts {
            let oi = c.open_interest as f64;
            if c.option_type.eq_ignore_ascii_case("call") {
                if cand_strike > c.strike {
                    total_payout += (cand_strike - c.strike) * oi;
                }
            } else if c.option_type.eq_ignore_ascii_case("put") && cand_strike < c.strike {
                total_payout += (c.strike - cand_strike) * oi;
            }
        }

        if total_payout < min_payout {
            min_payout = total_payout;
            max_pain_strike = cand_strike;
        }
    }

    max_pain_strike
}

/// Calculates Put/Call Ratio based on volume.
pub fn calculate_put_call_ratio(contracts: &[OptionContractData]) -> f64 {
    let mut call_vol: u64 = 0;
    let mut put_vol: u64 = 0;

    for c in contracts {
        if c.option_type.eq_ignore_ascii_case("call") {
            call_vol += c.volume;
        } else if c.option_type.eq_ignore_ascii_case("put") {
            put_vol += c.volume;
        }
    }

    if call_vol == 0 {
        0.0
    } else {
        put_vol as f64 / call_vol as f64
    }
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
    greeks: Option<DeribitGreeks>,
}

#[derive(Debug, serde::Deserialize)]
struct DeribitGreeks {
    delta: Option<f64>,
    gamma: Option<f64>,
    theta: Option<f64>,
    vega: Option<f64>,
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

        let (expiration_date, time_years) = match parse_deribit_exp(exp_str) {
            Some(v) => v,
            None => (format!("{}-exp", exp_str), 0.08),
        };

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

        let (delta, gamma, theta, vega) = if let Some(g) = &item.greeks {
            (
                g.delta.unwrap_or(0.0),
                g.gamma.unwrap_or(0.0),
                g.theta.unwrap_or(0.0),
                g.vega.unwrap_or(0.0),
            )
        } else {
            calculate_greeks(option_type, strike, underlying_price, time_years, iv, 0.045)
        };

        let open_interest = item.open_interest.unwrap_or(0.0).max(0.0) as u64;
        let volume = item.volume.unwrap_or(0.0).max(0.0) as u64;
        let is_call = option_type == "call";
        let gex = calculate_gex(gamma, underlying_price, open_interest as f64, is_call);

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
            delta,
            gamma,
            theta,
            vega,
            gex,
            open_interest,
            volume,
        });
    }

    Ok((underlying_price, contracts))
}

// Yahoo Finance JSON responses
#[derive(Debug, serde::Deserialize)]
struct YahooOptionsResponse {
    #[serde(rename = "optionChain")]
    option_chain: Option<YahooOptionChainResult>,
}

#[derive(Debug, serde::Deserialize)]
struct YahooOptionChainResult {
    result: Option<Vec<YahooOptionChain>>,
}

#[derive(Debug, serde::Deserialize)]
struct YahooOptionChain {
    quote: Option<YahooQuote>,
    options: Option<Vec<YahooOptionsList>>,
}

#[derive(Debug, serde::Deserialize)]
struct YahooQuote {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
struct YahooOptionsList {
    calls: Option<Vec<YahooOptionContract>>,
    puts: Option<Vec<YahooOptionContract>>,
}

#[derive(Debug, serde::Deserialize)]
struct YahooOptionContract {
    #[serde(rename = "contractSymbol")]
    contract_symbol: String,
    strike: f64,
    #[serde(rename = "lastPrice")]
    last_price: Option<f64>,
    bid: Option<f64>,
    ask: Option<f64>,
    #[serde(rename = "impliedVolatility")]
    implied_volatility: Option<f64>,
    #[serde(rename = "openInterest")]
    open_interest: Option<u64>,
    volume: Option<u64>,
    expiration: Option<i64>,
}

pub async fn fetch_yahoo_options(
    client: &reqwest::Client,
    symbol: &str,
) -> anyhow::Result<(f64, Vec<OptionContractData>)> {
    let url = format!(
        "https://query2.finance.yahoo.com/v7/finance/options/{}",
        symbol
    );
    let res: YahooOptionsResponse = client.get(&url).send().await?.json().await?;

    let chain = res
        .option_chain
        .and_then(|c| c.result)
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| anyhow::anyhow!("empty yahoo options response"))?;

    let underlying_price = chain
        .quote
        .and_then(|q| q.regular_market_price)
        .unwrap_or(0.0);

    let mut contracts = Vec::new();
    let now_ts = Utc::now().timestamp();

    if let Some(opts_lists) = chain.options {
        for opts in opts_lists {
            if let Some(calls) = opts.calls {
                for c in calls {
                    let exp_ts = c.expiration.unwrap_or(now_ts + 30 * 86400);
                    let exp_date = chrono::DateTime::from_timestamp(exp_ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "2026-07-25".to_string());
                    let time_years = ((exp_ts - now_ts) as f64 / (365.25 * 86400.0)).max(0.001);
                    let iv = c.implied_volatility.unwrap_or(0.2);
                    let (delta, gamma, theta, vega) =
                        calculate_greeks("call", c.strike, underlying_price, time_years, iv, 0.045);

                    let oi = c.open_interest.unwrap_or(0);
                    let vol = c.volume.unwrap_or(0);
                    let gex = calculate_gex(gamma, underlying_price, oi as f64, true);

                    contracts.push(OptionContractData {
                        contract_symbol: c.contract_symbol,
                        symbol: symbol.to_uppercase(),
                        option_type: "call".to_string(),
                        strike: c.strike,
                        expiration_date: exp_date,
                        mark_price: c.last_price.unwrap_or(0.0),
                        bid: c.bid,
                        ask: c.ask,
                        implied_volatility: iv,
                        delta,
                        gamma,
                        theta,
                        vega,
                        gex,
                        open_interest: oi,
                        volume: vol,
                    });
                }
            }

            if let Some(puts) = opts.puts {
                for p in puts {
                    let exp_ts = p.expiration.unwrap_or(now_ts + 30 * 86400);
                    let exp_date = chrono::DateTime::from_timestamp(exp_ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "2026-07-25".to_string());
                    let time_years = ((exp_ts - now_ts) as f64 / (365.25 * 86400.0)).max(0.001);
                    let iv = p.implied_volatility.unwrap_or(0.2);
                    let (delta, gamma, theta, vega) =
                        calculate_greeks("put", p.strike, underlying_price, time_years, iv, 0.045);

                    let oi = p.open_interest.unwrap_or(0);
                    let vol = p.volume.unwrap_or(0);
                    let gex = calculate_gex(gamma, underlying_price, oi as f64, false);

                    contracts.push(OptionContractData {
                        contract_symbol: p.contract_symbol,
                        symbol: symbol.to_uppercase(),
                        option_type: "put".to_string(),
                        strike: p.strike,
                        expiration_date: exp_date,
                        mark_price: p.last_price.unwrap_or(0.0),
                        bid: p.bid,
                        ask: p.ask,
                        implied_volatility: iv,
                        delta,
                        gamma,
                        theta,
                        vega,
                        gex,
                        open_interest: oi,
                        volume: vol,
                    });
                }
            }
        }
    }

    Ok((underlying_price, contracts))
}

async fn publish_options_data(
    broker: &dyn EventPublisher,
    symbol: &str,
    underlying_price: f64,
    contracts: &[OptionContractData],
) {
    let pcr = calculate_put_call_ratio(contracts);
    let max_pain = calculate_max_pain(contracts);
    let total_oi: u64 = contracts.iter().map(|c| c.open_interest).sum();
    let total_vol: u64 = contracts.iter().map(|c| c.volume).sum();
    let total_gex: f64 = contracts.iter().map(|c| c.gex).sum();

    let iv_atm = contracts
        .iter()
        .min_by(|a, b| {
            let diff_a = (a.strike - underlying_price).abs();
            let diff_b = (b.strike - underlying_price).abs();
            diff_a
                .partial_cmp(&diff_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.implied_volatility)
        .unwrap_or(0.0);

    let summary_payload = json!({
        "id": symbol,
        "symbol": symbol,
        "underlying_price": underlying_price,
        "put_call_ratio": pcr,
        "max_pain_strike": max_pain,
        "total_open_interest": total_oi,
        "total_volume": total_vol,
        "total_gex": total_gex,
        "iv_atm": iv_atm,
        "updated_at": Utc::now().to_rfc3339(),
    });

    let chain_payload = json!({
        "symbol": symbol,
        "underlying_price": underlying_price,
        "contracts": contracts,
        "updated_at": Utc::now().to_rfc3339(),
    });

    if let Err(e) = broker
        .publish_str(
            subjects::MARKET_OPTIONS_SUMMARY_V1,
            &summary_payload.to_string(),
        )
        .await
    {
        error!(symbol = symbol, error = %e, "failed to publish market.options.summary");
    }

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

        for symbol in &["SPY", "QQQ", "AAPL", "MSFT", "TSLA", "NVDA"] {
            match fetch_yahoo_options(&client, symbol).await {
                Ok((underlying_price, contracts)) => {
                    if !contracts.is_empty() {
                        publish_options_data(&*broker, symbol, underlying_price, &contracts).await;
                    }
                }
                Err(e) => {
                    error!(worker = "options_feed", symbol = symbol, error = %e, "failed to fetch Yahoo options");
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
    fn test_calculate_greeks_call_and_put() {
        let (delta_c, gamma_c, theta_c, vega_c) =
            calculate_greeks("call", 100.0, 100.0, 1.0, 0.2, 0.05);
        assert!(delta_c > 0.5 && delta_c < 0.7, "delta call was {}", delta_c);
        assert!(gamma_c > 0.0, "gamma call was {}", gamma_c);
        assert!(theta_c < 0.0, "theta call was {}", theta_c);
        assert!(vega_c > 0.0, "vega call was {}", vega_c);

        let (delta_p, gamma_p, theta_p, vega_p) =
            calculate_greeks("put", 100.0, 100.0, 1.0, 0.2, 0.05);
        assert!(delta_p < 0.0 && delta_p > -0.5, "delta put was {}", delta_p);
        assert!(
            (gamma_c - gamma_p).abs() < 1e-6,
            "gamma put should match call gamma"
        );
        assert!(
            (vega_c - vega_p).abs() < 1e-6,
            "vega put should match call vega"
        );
        assert!(theta_p < 0.0, "theta put was {}", theta_p);
    }

    #[test]
    fn test_calculate_gex() {
        let call_gex = calculate_gex(0.02, 100.0, 50.0, true);
        assert_eq!(call_gex, 1000000.0);

        let put_gex = calculate_gex(0.02, 100.0, 50.0, false);
        assert_eq!(put_gex, -1000000.0);
    }

    #[test]
    fn test_calculate_max_pain() {
        let contracts = vec![
            OptionContractData {
                contract_symbol: "CALL-90".to_string(),
                symbol: "TEST".to_string(),
                option_type: "call".to_string(),
                strike: 90.0,
                expiration_date: "2026-07-25".to_string(),
                mark_price: 10.0,
                bid: None,
                ask: None,
                implied_volatility: 0.2,
                delta: 0.8,
                gamma: 0.01,
                theta: -0.01,
                vega: 0.1,
                gex: 1000.0,
                open_interest: 100,
                volume: 50,
            },
            OptionContractData {
                contract_symbol: "CALL-100".to_string(),
                symbol: "TEST".to_string(),
                option_type: "call".to_string(),
                strike: 100.0,
                expiration_date: "2026-07-25".to_string(),
                mark_price: 5.0,
                bid: None,
                ask: None,
                implied_volatility: 0.2,
                delta: 0.5,
                gamma: 0.02,
                theta: -0.02,
                vega: 0.2,
                gex: 2000.0,
                open_interest: 1000,
                volume: 500,
            },
            OptionContractData {
                contract_symbol: "PUT-110".to_string(),
                symbol: "TEST".to_string(),
                option_type: "put".to_string(),
                strike: 110.0,
                expiration_date: "2026-07-25".to_string(),
                mark_price: 12.0,
                bid: None,
                ask: None,
                implied_volatility: 0.2,
                delta: -0.7,
                gamma: 0.01,
                theta: -0.01,
                vega: 0.15,
                gex: -1500.0,
                open_interest: 1000,
                volume: 400,
            },
        ];

        let max_pain = calculate_max_pain(&contracts);
        // At 90: Put 110 pays out (110-90)*1000 = 20,000
        // At 100: Call 90 pays (100-90)*100 = 1,000; Put 110 pays (110-100)*1000 = 10,000 => total 11,000
        // At 110: Call 90 pays (110-90)*100 = 2,000; Call 100 pays (110-100)*1000 = 10,000 => total 12,000
        // Min payout is at 100 (11,000)
        assert_eq!(max_pain, 100.0);
    }

    #[test]
    fn test_calculate_put_call_ratio() {
        let contracts = vec![
            OptionContractData {
                contract_symbol: "CALL-1".to_string(),
                symbol: "TEST".to_string(),
                option_type: "call".to_string(),
                strike: 100.0,
                expiration_date: "2026-07-25".to_string(),
                mark_price: 5.0,
                bid: None,
                ask: None,
                implied_volatility: 0.2,
                delta: 0.5,
                gamma: 0.02,
                theta: -0.02,
                vega: 0.2,
                gex: 2000.0,
                open_interest: 100,
                volume: 200,
            },
            OptionContractData {
                contract_symbol: "PUT-1".to_string(),
                symbol: "TEST".to_string(),
                option_type: "put".to_string(),
                strike: 100.0,
                expiration_date: "2026-07-25".to_string(),
                mark_price: 5.0,
                bid: None,
                ask: None,
                implied_volatility: 0.2,
                delta: -0.5,
                gamma: 0.02,
                theta: -0.02,
                vega: 0.2,
                gex: -2000.0,
                open_interest: 100,
                volume: 300,
            },
        ];

        let pcr = calculate_put_call_ratio(&contracts);
        assert_eq!(pcr, 1.5);
    }

    #[test]
    fn test_parse_deribit_exp() {
        let res = parse_deribit_exp("26JUL26");
        assert!(res.is_some());
        let (date_str, _time_years) = res.unwrap();
        assert_eq!(date_str, "2026-07-26");
    }
}
