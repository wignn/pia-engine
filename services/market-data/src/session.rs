use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use serde::Serialize;

use crate::{calendar::CalendarCache, state::AppState};

#[derive(Debug, Clone, Serialize)]
pub struct MarketSessionStatus {
    pub exchange: String,
    pub timezone: String,
    pub state: String,
    pub is_open: bool,
    pub reason: String,
}

impl MarketSessionStatus {
    fn new(exchange: &str, timezone: &str, state: &str, is_open: bool, reason: &str) -> Self {
        Self {
            exchange: exchange.to_string(),
            timezone: timezone.to_string(),
            state: state.to_string(),
            is_open,
            reason: reason.to_string(),
        }
    }
}

pub async fn get_session(
    Path(symbol): Path<String>,
    State(state): State<AppState>,
) -> Json<MarketSessionStatus> {
    let asset_type = state
        .calendar
        .exchange_for_symbol(&symbol)
        .map(|mapping| mapping.asset_type)
        .unwrap_or_else(|| "unknown".to_string());
    Json(session_status(
        &symbol,
        &asset_type,
        Utc::now(),
        Some(&state.calendar),
    ))
}

pub fn session_status(
    symbol: &str,
    asset_type: &str,
    now: DateTime<Utc>,
    calendar: Option<&CalendarCache>,
) -> MarketSessionStatus {
    let sym = symbol.to_uppercase();
    let asset = asset_type.to_lowercase();

    if asset == "crypto" || sym.ends_with("USDT") {
        return MarketSessionStatus::new("CRYPTO", "UTC", "open", true, "crypto trades 24/7");
    }

    if sym == "XAUUSD" || asset == "forex" || is_fx_symbol(&sym) {
        return forex_session(now);
    }

    if let Some(status) = calendar.and_then(|cache| calendar_session(cache, &sym, &asset, now)) {
        return status;
    }

    fallback_session(&sym, &asset, now)
}

fn calendar_session(
    cache: &CalendarCache,
    symbol: &str,
    asset_type: &str,
    now: DateTime<Utc>,
) -> Option<MarketSessionStatus> {
    let mapped = cache.exchange_for_symbol(symbol)?;
    if mapped.asset_type == "crypto" || asset_type == "crypto" {
        return Some(MarketSessionStatus::new(
            "CRYPTO",
            "UTC",
            "open",
            true,
            "crypto trades 24/7",
        ));
    }

    let rule = cache.exchange_rule(&mapped.exchange_code)?;
    let tz: Tz = rule.timezone.parse().ok()?;
    let local = now.with_timezone(&tz);
    let day = weekday_code(local.weekday());

    if !rule.working_days.contains(day) {
        return Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "closed",
            false,
            "non-working day",
        ));
    }

    if cache.is_holiday(&rule.exchange_code, local.date_naive()) {
        return Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "holiday",
            false,
            "market holiday",
        ));
    }

    let Some(open) = rule.regular_open else {
        return Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "unknown",
            true,
            "calendar has no open time",
        ));
    };
    let Some(close) = rule.regular_close else {
        return Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "unknown",
            true,
            "calendar has no close time",
        ));
    };

    let time = local.time();
    if time >= open && time < close {
        Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "open",
            true,
            "regular session",
        ))
    } else {
        Some(MarketSessionStatus::new(
            &rule.exchange_code,
            &rule.timezone,
            "closed",
            false,
            "outside regular session",
        ))
    }
}

fn fallback_session(symbol: &str, asset_type: &str, now: DateTime<Utc>) -> MarketSessionStatus {
    if is_us_symbol(symbol) {
        return regular_session(
            "US",
            chrono_tz::America::New_York,
            now,
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
    }

    if asset_type == "stock" || is_idx_symbol(symbol) {
        return regular_session(
            "IDX",
            chrono_tz::Asia::Jakarta,
            now,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            Some((
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
            )),
        );
    }

    if matches!(symbol, "SPX" | "DXY") {
        return regular_session(
            "US",
            chrono_tz::America::New_York,
            now,
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
    }

    match symbol {
        "N225" => {
            return regular_session(
                "JP",
                chrono_tz::Asia::Tokyo,
                now,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
                None,
            )
        }
        "HSI" => {
            return regular_session(
                "HK",
                chrono_tz::Asia::Hong_Kong,
                now,
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                None,
            )
        }
        "SSEC" => {
            return regular_session(
                "CN",
                chrono_tz::Asia::Shanghai,
                now,
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
                None,
            )
        }
        "KOSPI" => {
            return regular_session(
                "KR",
                chrono_tz::Asia::Seoul,
                now,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
                None,
            )
        }
        "STI" => {
            return regular_session(
                "SG",
                chrono_tz::Asia::Singapore,
                now,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                None,
            )
        }
        "JCI" => {
            return regular_session(
                "IDX",
                chrono_tz::Asia::Jakarta,
                now,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                Some((
                    NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(13, 30, 0).unwrap(),
                )),
            )
        }
        "ASX200" => {
            return regular_session(
                "AU",
                chrono_tz::Australia::Sydney,
                now,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                None,
            )
        }
        "NIFTY50" | "SENSEX" => {
            return regular_session(
                "IN",
                chrono_tz::Asia::Kolkata,
                now,
                NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
                None,
            )
        }
        _ => {}
    }

    MarketSessionStatus::new("UNKNOWN", "UTC", "unknown", true, "unknown market session")
}

fn forex_session(now: DateTime<Utc>) -> MarketSessionStatus {
    let weekday = now.weekday();
    let hour = now.hour();
    let minute = now.minute();

    if weekday == Weekday::Sat
        || (weekday == Weekday::Fri && hour >= 22)
        || (weekday == Weekday::Sun && hour < 22)
    {
        return MarketSessionStatus::new("FX", "UTC", "closed", false, "forex weekend close");
    }

    if hour == 22 || (hour == 23 && minute < 5) {
        return MarketSessionStatus::new(
            "FX",
            "UTC",
            "break",
            false,
            "daily forex rollover / liquidity break",
        );
    }

    MarketSessionStatus::new("FX", "UTC", "open", true, "regular forex session")
}

fn regular_session(
    exchange: &str,
    tz: Tz,
    now: DateTime<Utc>,
    open: NaiveTime,
    close: NaiveTime,
    break_window: Option<(NaiveTime, NaiveTime)>,
) -> MarketSessionStatus {
    let local = now.with_timezone(&tz);
    let weekday = local.weekday();
    if weekday == Weekday::Sat || weekday == Weekday::Sun {
        return MarketSessionStatus::new(exchange, tz.name(), "closed", false, "weekend close");
    }

    let time = local.time();
    if let Some((break_start, break_end)) = break_window {
        if time >= break_start && time < break_end {
            return MarketSessionStatus::new(
                exchange,
                tz.name(),
                "break",
                false,
                "intraday market break",
            );
        }
    }

    if time >= open && time < close {
        MarketSessionStatus::new(exchange, tz.name(), "open", true, "regular session")
    } else {
        MarketSessionStatus::new(
            exchange,
            tz.name(),
            "closed",
            false,
            "outside regular session",
        )
    }
}

fn weekday_code(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

fn is_fx_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "EURUSD" | "GBPUSD" | "USDJPY" | "AUDUSD" | "NZDUSD" | "USDCAD" | "USDCHF" | "USDSGD"
    )
}

fn is_us_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "SPX"
            | "DXY"
            | "AAPL"
            | "MSFT"
            | "NVDA"
            | "GOOGL"
            | "META"
            | "AMZN"
            | "TSLA"
            | "AVGO"
            | "BRKB"
            | "JPM"
            | "V"
            | "LLY"
            | "WMT"
            | "UNH"
            | "COST"
    )
}

fn is_idx_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "BBCA" | "BBRI" | "BMRI" | "TLKM" | "ASII" | "UNVR" | "ICBP" | "BBNI" | "ADRO" | "MDKA"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn crypto_is_always_open() {
        let now = Utc.with_ymd_and_hms(2026, 5, 30, 12, 0, 0).unwrap();
        assert!(session_status("BTCUSDT", "crypto", now, None).is_open);
    }

    #[test]
    fn forex_has_daily_break() {
        let now = Utc.with_ymd_and_hms(2026, 5, 25, 22, 30, 0).unwrap();
        let status = session_status("EURUSD", "forex", now, None);
        assert_eq!(status.state, "break");
        assert!(!status.is_open);
    }

    #[test]
    fn idx_closes_after_regular_session() {
        let now = Utc.with_ymd_and_hms(2026, 5, 25, 12, 0, 0).unwrap();
        let status = session_status("BBRI", "stock", now, None);
        assert_eq!(status.state, "closed");
    }

    #[test]
    fn us_index_closes_after_regular_session() {
        let now = Utc.with_ymd_and_hms(2026, 5, 27, 22, 0, 0).unwrap();
        let status = session_status("SPX", "index", now, None);
        assert_eq!(status.exchange, "US");
        assert!(!status.is_open);
    }
}
