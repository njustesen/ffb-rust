/// 1:1 translation of `com.fumbbl.ffb.util.DateTool`.
///
/// The Java source uses `java.util.Date` + `SimpleDateFormat`.  In Rust we
/// represent timestamps as milliseconds since the Unix epoch (i64), matching
/// the internal representation used by `java.util.Date.getTime()`.
pub struct DateTool;

/// Java: `DateTool.TIMESTAMP_FORMAT` = `"yyyy-MM-dd HH:mm:ss.SSS"`.
pub const TIMESTAMP_FORMAT: &str = "yyyy-MM-dd HH:mm:ss.SSS";

impl DateTool {
    /// Java: `isEqual(Date, Date)`.
    pub fn is_equal(date1: Option<i64>, date2: Option<i64>) -> bool {
        date1 == date2
    }

    /// Java: `formatTimestamp(Date)` — format epoch-millis as `YYYY-MM-DD HH:MM:SS.mmm`.
    pub fn format_timestamp(millis: i64) -> String {
        let secs = millis / 1000;
        let ms = (millis % 1000).unsigned_abs();
        let mins = secs / 60 % 60;
        let hours = secs / 3600 % 24;
        let days_from_epoch = secs / 86400;
        // Simplified gregorian calendar (good enough for tests / non-display use)
        let (y, mo, d) = Self::days_to_ymd(days_from_epoch);
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}", y, mo, d, hours, mins, secs % 60, ms)
    }

    /// Java: `parseTimestamp(String)` — parse the formatted string back to epoch-millis.
    /// Returns `None` on parse failure (Java throws FantasyFootballException).
    pub fn parse_timestamp(s: &str) -> Option<i64> {
        // Expected: "YYYY-MM-DD HH:MM:SS.mmm"
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
        if parts.len() != 2 { return None; }
        let date_parts: Vec<i64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
        let time_s: Vec<&str> = parts[1].splitn(2, '.').collect();
        let time_parts: Vec<i64> = time_s[0].split(':').filter_map(|p| p.parse().ok()).collect();
        if date_parts.len() != 3 || time_parts.len() != 3 { return None; }
        let ms: i64 = if time_s.len() == 2 { time_s[1].parse().ok()? } else { 0 };
        let days = Self::ymd_to_days(date_parts[0], date_parts[1] as u32, date_parts[2] as u32);
        let total_secs = days * 86400 + time_parts[0] * 3600 + time_parts[1] * 60 + time_parts[2];
        Some(total_secs * 1000 + ms)
    }

    // Gregorian calendar helpers (proleptic, sufficient for game timestamps).
    fn days_to_ymd(days: i64) -> (i64, u32, u32) {
        // Shift epoch to 1 Mar 0000 for simpler leap-year arithmetic.
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        (y, m as u32, d as u32)
    }

    fn ymd_to_days(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let doy = (153 * (m as i64 + if m > 2 { -3 } else { 9 }) + 2) / 5 + d as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_equal_same() {
        assert!(DateTool::is_equal(Some(1000), Some(1000)));
    }

    #[test]
    fn is_equal_none_both() {
        assert!(DateTool::is_equal(None, None));
    }

    #[test]
    fn format_and_parse_roundtrip() {
        let ts = 1_700_000_000_000i64;
        let s = DateTool::format_timestamp(ts);
        let parsed = DateTool::parse_timestamp(&s);
        assert_eq!(parsed, Some(ts));
    }
}
