use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MarketSessionStatus {
    pub exchange: &'static str,
    pub timezone: &'static str,
    pub state: &'static str,
    pub is_open: bool,
    pub reason: &'static str,
}

pub fn session_status(symbol: &str, asset_type: &str, now: DateTime<Utc>) -> MarketSessionStatus {
    let sym = symbol.to_uppercase();
    let asset = asset_type.to_lowercase();

    if asset == "crypto" || sym.ends_with("USDT") {
        return MarketSessionStatus {
            exchange: "CRYPTO",
            timezone: "UTC",
            state: "open",
            is_open: true,
            reason: "crypto trades 24/7",
        };
    }

    if sym == "XAUUSD" || asset == "forex" || is_fx_symbol(&sym) {
        return forex_session(now);
    }

    if asset == "stock" || is_idx_symbol(&sym) {
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

    if matches!(sym.as_str(), "SPX" | "DXY") {
        return regular_session(
            "US",
            chrono_tz::America::New_York,
            now,
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
    }

    if matches!(
        sym.as_str(),
        "N225" | "HSI" | "SSEC" | "KOSPI" | "STI" | "JCI" | "ASX200" | "NIFTY50" | "SENSEX"
    ) {
        return regular_session(
            "INDEX",
            chrono_tz::Asia::Singapore,
            now,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            None,
        );
    }

    MarketSessionStatus {
        exchange: "UNKNOWN",
        timezone: "UTC",
        state: "unknown",
        is_open: true,
        reason: "unknown market session",
    }
}

fn forex_session(now: DateTime<Utc>) -> MarketSessionStatus {
    let weekday = now.weekday();
    let hour = now.hour();
    let minute = now.minute();

    if weekday == Weekday::Sat
        || (weekday == Weekday::Fri && hour >= 22)
        || (weekday == Weekday::Sun && hour < 22)
    {
        return MarketSessionStatus {
            exchange: "FX",
            timezone: "UTC",
            state: "closed",
            is_open: false,
            reason: "forex weekend close",
        };
    }

    if hour == 22 || (hour == 23 && minute < 5) {
        return MarketSessionStatus {
            exchange: "FX",
            timezone: "UTC",
            state: "break",
            is_open: false,
            reason: "daily forex rollover / liquidity break",
        };
    }

    MarketSessionStatus {
        exchange: "FX",
        timezone: "UTC",
        state: "open",
        is_open: true,
        reason: "regular forex session",
    }
}

fn regular_session(
    exchange: &'static str,
    tz: Tz,
    now: DateTime<Utc>,
    open: NaiveTime,
    close: NaiveTime,
    break_window: Option<(NaiveTime, NaiveTime)>,
) -> MarketSessionStatus {
    let local = now.with_timezone(&tz);
    let weekday = local.weekday();
    if weekday == Weekday::Sat || weekday == Weekday::Sun {
        return MarketSessionStatus {
            exchange,
            timezone: tz.name(),
            state: "closed",
            is_open: false,
            reason: "weekend close",
        };
    }

    let time = local.time();
    if let Some((break_start, break_end)) = break_window {
        if time >= break_start && time < break_end {
            return MarketSessionStatus {
                exchange,
                timezone: tz.name(),
                state: "break",
                is_open: false,
                reason: "intraday market break",
            };
        }
    }

    if time >= open && time < close {
        MarketSessionStatus {
            exchange,
            timezone: tz.name(),
            state: "open",
            is_open: true,
            reason: "regular session",
        }
    } else {
        MarketSessionStatus {
            exchange,
            timezone: tz.name(),
            state: "closed",
            is_open: false,
            reason: "outside regular session",
        }
    }
}

fn is_fx_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "EURUSD" | "GBPUSD" | "USDJPY" | "AUDUSD" | "NZDUSD" | "USDCAD" | "USDCHF" | "USDSGD"
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
        assert!(session_status("BTCUSDT", "crypto", now).is_open);
    }

    #[test]
    fn forex_has_daily_break() {
        let now = Utc.with_ymd_and_hms(2026, 5, 25, 22, 30, 0).unwrap();
        let status = session_status("EURUSD", "forex", now);
        assert_eq!(status.state, "break");
        assert!(!status.is_open);
    }

    #[test]
    fn idx_closes_after_regular_session() {
        let now = Utc.with_ymd_and_hms(2026, 5, 25, 12, 0, 0).unwrap();
        let status = session_status("BBRI", "stock", now);
        assert_eq!(status.state, "closed");
    }
}
