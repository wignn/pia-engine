use chrono::{Datelike, Timelike, Utc, Weekday};
use std::time::Duration;

/// Check whether the Forex market is currently open.
///
/// Forex market hours (simplified):
/// - **Closed**: Friday 22:00 UTC → Sunday 22:00 UTC
/// - **Open**: all other times
///
/// This means:
/// - Friday after 22:00 UTC → closed
/// - Saturday all day → closed  
/// - Sunday before 22:00 UTC → closed
/// - Sunday after 22:00 UTC → open
/// - Monday–Friday before 22:00 → open
pub fn is_market_open() -> bool {
    is_market_open_at(Utc::now())
}

/// Testable version — checks market status at a specific UTC time.
pub fn is_market_open_at(now: chrono::DateTime<Utc>) -> bool {
    let weekday = now.weekday();
    let hour = now.hour();

    match weekday {
        // Saturday: always closed
        Weekday::Sat => false,
        // Sunday: open only after 22:00 UTC
        Weekday::Sun => hour >= 22,
        // Friday: open only before 22:00 UTC
        Weekday::Fri => hour < 22,
        // Mon–Thu: always open
        _ => true,
    }
}

/// Calculate how long until the market opens next.
///
/// Returns `Duration::ZERO` if the market is already open.
pub fn duration_until_next_open() -> Duration {
    duration_until_next_open_from(Utc::now())
}

/// Testable version of `duration_until_next_open`.
pub fn duration_until_next_open_from(now: chrono::DateTime<Utc>) -> Duration {
    if is_market_open_at(now) {
        return Duration::ZERO;
    }

    // Market opens on Sunday 22:00 UTC.
    // Calculate how many seconds until next Sunday 22:00 UTC.
    let weekday = now.weekday();
    let hour = now.hour();
    let minute = now.minute();
    let second = now.second();

    let current_seconds_in_day = (hour * 3600 + minute * 60 + second) as i64;
    let target_seconds_in_day: i64 = 22 * 3600; // 22:00:00

    let days_until_sunday = match weekday {
        Weekday::Fri => 2, // Fri → Sun
        Weekday::Sat => 1, // Sat → Sun
        Weekday::Sun => 0, // Sun → Sun (same day, but before 22:00)
        _ => {
            // Should not reach here if market is closed, but handle gracefully
            // Mon=0 days would be wrong, let's compute correctly
            // If we're here, market is closed, which only happens Fri 22+, Sat, Sun <22
            0
        }
    };

    let remaining_today = 86400 - current_seconds_in_day;

    if days_until_sunday == 0 {
        // We're on Sunday before 22:00
        let secs = target_seconds_in_day - current_seconds_in_day;
        if secs > 0 {
            return Duration::from_secs(secs as u64);
        }
        return Duration::ZERO;
    }

    // Full days remaining + time on Sunday until 22:00
    let total_secs = remaining_today
        + (days_until_sunday - 1) * 86400
        + target_seconds_in_day;

    Duration::from_secs(total_secs.max(0) as u64)
}

/// Calculate how long until the market closes next.
///
/// Returns `None` if the market is currently closed.
/// Market closes on Friday at 22:00 UTC.
pub fn duration_until_close() -> Option<Duration> {
    duration_until_close_from(Utc::now())
}

/// Testable version of `duration_until_close`.
pub fn duration_until_close_from(now: chrono::DateTime<Utc>) -> Option<Duration> {
    if !is_market_open_at(now) {
        return None;
    }

    let weekday = now.weekday();
    let hour = now.hour();
    let minute = now.minute();
    let second = now.second();

    let current_seconds_in_day = (hour * 3600 + minute * 60 + second) as i64;
    let target_seconds_in_day: i64 = 22 * 3600;

    // Days until Friday
    let days_until_friday = match weekday {
        Weekday::Sun => {
            // If Sunday >= 22:00, market is open, next close is Friday
            5
        }
        Weekday::Mon => 4,
        Weekday::Tue => 3,
        Weekday::Wed => 2,
        Weekday::Thu => 1,
        Weekday::Fri => 0,
        Weekday::Sat => {
            // Should not be open on Saturday
            return None;
        }
    };

    if days_until_friday == 0 {
        // We're on Friday, close at 22:00
        let secs = target_seconds_in_day - current_seconds_in_day;
        if secs > 0 {
            return Some(Duration::from_secs(secs as u64));
        }
        // Past 22:00 on Friday — should be closed, but guard
        return None;
    }

    // Full days remaining + time on Friday until 22:00
    let remaining_today = 86400 - current_seconds_in_day;
    let total_secs = remaining_today
        + (days_until_friday - 1) * 86400
        + target_seconds_in_day;

    Some(Duration::from_secs(total_secs.max(0) as u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(year: i32, month: u32, day: u32, hour: u32, min: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, min, 0).unwrap()
    }

    #[test]
    fn monday_is_open() {
        // 2026-05-04 is a Monday
        assert!(is_market_open_at(utc(2026, 5, 4, 12, 0)));
    }

    #[test]
    fn friday_before_close_is_open() {
        // Friday 21:59 UTC → open
        assert!(is_market_open_at(utc(2026, 5, 8, 21, 59)));
    }

    #[test]
    fn friday_after_close_is_closed() {
        // Friday 22:00 UTC → closed
        assert!(!is_market_open_at(utc(2026, 5, 8, 22, 0)));
        // Friday 23:00 UTC → closed
        assert!(!is_market_open_at(utc(2026, 5, 8, 23, 0)));
    }

    #[test]
    fn saturday_is_closed() {
        assert!(!is_market_open_at(utc(2026, 5, 9, 12, 0)));
    }

    #[test]
    fn sunday_before_open_is_closed() {
        // Sunday 21:59 UTC → closed
        assert!(!is_market_open_at(utc(2026, 5, 10, 21, 59)));
    }

    #[test]
    fn sunday_after_open_is_open() {
        // Sunday 22:00 UTC → open
        assert!(is_market_open_at(utc(2026, 5, 10, 22, 0)));
        // Sunday 23:00 UTC → open
        assert!(is_market_open_at(utc(2026, 5, 10, 23, 0)));
    }

    #[test]
    fn duration_until_open_on_saturday() {
        // Saturday 12:00 → should wait until Sunday 22:00
        let sat = utc(2026, 5, 9, 12, 0);
        let dur = duration_until_next_open_from(sat);
        // 12 hours remaining Saturday + 22 hours Sunday = 34 hours
        assert_eq!(dur, Duration::from_secs(34 * 3600));
    }

    #[test]
    fn duration_until_open_when_already_open() {
        let mon = utc(2026, 5, 4, 12, 0);
        let dur = duration_until_next_open_from(mon);
        assert_eq!(dur, Duration::ZERO);
    }

    #[test]
    fn duration_until_close_on_monday() {
        // Monday 12:00 → close Friday 22:00
        // remaining Monday: 12h, Tue: 24h, Wed: 24h, Thu: 24h, Fri until 22:00: 22h
        // Total: 12 + 24 + 24 + 24 + 22 = 106 hours
        let mon = utc(2026, 5, 4, 12, 0);
        let dur = duration_until_close_from(mon);
        assert_eq!(dur, Some(Duration::from_secs(106 * 3600)));
    }

    #[test]
    fn duration_until_close_when_closed() {
        let sat = utc(2026, 5, 9, 12, 0);
        assert_eq!(duration_until_close_from(sat), None);
    }
}
