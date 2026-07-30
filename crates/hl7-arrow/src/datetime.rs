//! Minimal HL7 v2 `TS` (timestamp) parsing: `YYYY[MM[DD[HH[MM[SS[.S...]]]]]]`,
//! with an optional trailing `+/-ZZZZ` timezone offset (ignored — the
//! ORU^R01 schema stores naive `Timestamp(Microsecond, None)` values).
//!
//! Written by hand instead of pulling in `chrono`: `hl7-arrow` depends only
//! on `hl7-v2` and `arrow` by design (see crate README), and converting a
//! civil date to days-since-epoch is a small, well-known integer
//! computation (Howard Hinnant's `days_from_civil` algorithm).

/// Parse an HL7 `TS` value into microseconds since the Unix epoch (UTC-naive).
///
/// Returns `None` if `raw` is empty or not a valid HL7 timestamp.
pub(crate) fn parse_hl7_datetime(raw: &str) -> Option<i64> {
    if raw.is_empty() {
        return None;
    }

    // Strip a trailing timezone offset (`+HHMM` / `-HHMM`); position must be
    // past the date portion so a leading sign can't be misread.
    let without_tz = match raw[8.min(raw.len())..].find(['+', '-']) {
        Some(rel_idx) => &raw[..8 + rel_idx],
        None => raw,
    };

    let (date_time, frac_micros) = match without_tz.split_once('.') {
        Some((head, frac)) => (head, parse_fraction_micros(frac)),
        None => (without_tz, 0),
    };

    if date_time.len() < 8 || !date_time.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let year: i32 = date_time[0..4].parse().ok()?;
    let month: u32 = date_time[4..6].parse().ok()?;
    let day: u32 = date_time[6..8].parse().ok()?;
    let hour: i64 = digits(date_time, 8, 10)?;
    let minute: i64 = digits(date_time, 10, 12)?;
    let second: i64 = digits(date_time, 12, 14)?;

    let days = days_from_civil(year, month, day)?;
    let seconds_of_day = hour * 3600 + minute * 60 + second;
    Some(days * 86_400_000_000 + seconds_of_day * 1_000_000 + frac_micros)
}

fn digits(s: &str, start: usize, end: usize) -> Option<i64> {
    match s.get(start..end.min(s.len())) {
        Some(slice) if !slice.is_empty() => slice.parse().ok(),
        _ => Some(0),
    }
}

fn parse_fraction_micros(frac: &str) -> i64 {
    let mut digits: String = frac.chars().filter(char::is_ascii_digit).take(6).collect();
    while digits.len() < 6 {
        digits.push('0');
    }
    digits.parse().unwrap_or(0)
}

/// Days since 1970-01-01 for a proleptic-Gregorian civil date.
/// Howard Hinnant's `days_from_civil` (public domain).
fn days_from_civil(y: i32, m: u32, d: u32) -> Option<i64> {
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m as i64 + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    Some(era * 146097 + doe - 719468)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_date_only() {
        // 1980-06-15 00:00:00 UTC
        assert_eq!(parse_hl7_datetime("19800615"), Some(329_875_200_000_000));
    }

    #[test]
    fn parses_full_datetime() {
        // 2024-01-15 10:30:00 UTC
        assert_eq!(
            parse_hl7_datetime("20240115103000"),
            Some(1_705_314_600_000_000)
        );
    }

    #[test]
    fn parses_fractional_seconds() {
        assert_eq!(
            parse_hl7_datetime("20240115103000.5"),
            Some(1_705_314_600_500_000)
        );
    }

    #[test]
    fn strips_timezone_offset() {
        assert_eq!(
            parse_hl7_datetime("20240115103000+0700"),
            parse_hl7_datetime("20240115103000")
        );
    }

    #[test]
    fn rejects_empty_and_malformed() {
        assert_eq!(parse_hl7_datetime(""), None);
        assert_eq!(parse_hl7_datetime("not-a-date"), None);
        assert_eq!(parse_hl7_datetime("2024"), None);
    }
}
