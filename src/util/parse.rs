use chrono::{DateTime, Duration, NaiveDate, Utc};
use std::str::FromStr;

pub fn parse_since(s: &str) -> Result<DateTime<Utc>, String> {
    // RFC3339 timestamp
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Date without Time
    if let Ok(date) = NaiveDate::from_str(s) {
        let dt = date.and_hms_opt(0, 0, 0).ok_or("Invalid date")?;
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    }

    // Relative Times (7d, 24h, 30m)
    if let Some(num) = s.strip_suffix('d') {
        if let Ok(n) = num.parse::<i64>() {
            return Ok(Utc::now() - Duration::days(n));
        }
    }

    if let Some(num) = s.strip_suffix('h') {
        if let Ok(n) = num.parse::<i64>() {
            return Ok(Utc::now() - Duration::hours(n));
        }
    }

    if let Some(num) = s.strip_suffix('m') {
        if let Ok(n) = num.parse::<i64>() {
            return Ok(Utc::now() - Duration::minutes(n));
        }
    }

    // Keywords
    match s {
        "now" => return Ok(Utc::now()),
        "today" => {
            let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(today, Utc));
        }
        "yesterday" => {
            let yesterday = (Utc::now() - Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(yesterday, Utc));
        }
        _ => {}
    }

    Err(format!(
        "Invalid date '{}'. Supported formats: YYYY-MM-DD, RFC3339 timestamp, 7d, 24h, yesterday.",
        s
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    struct TestCase {
        input: &'static str,
        expected: DateTime<Utc>,
    }

    #[test]
    fn test_absolute_inputs() {
        let cases = vec![
            TestCase {
                input: "2026-02-06T12:00:00Z",
                expected: Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap(),
            },
            TestCase {
                input: "2026-02-06",
                expected: Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap(),
            },
            TestCase {
                input: "2026-02-06T13:00:00+01:00",
                expected: Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap(),
            },
        ];

        for case in cases {
            let result = parse_since(case.input).unwrap();
            assert_eq!(result, case.expected, "failed for input {}", case.input);
        }
    }

    #[test]
    fn test_relative_inputs() {
        let cases = vec![
            ("7d", Duration::days(7)),
            ("24h", Duration::hours(24)),
            ("30m", Duration::minutes(30)),
        ];

        for (input, duration) in cases {
            let before = Utc::now();
            let result = parse_since(input).unwrap();
            let after = Utc::now();

            let min = before - duration;
            let max = after - duration;

            assert!(result >= min && result <= max, "failed for input {}", input);
        }
    }

    #[test]
    fn test_keywords() {
        let now_before = Utc::now();
        let now = parse_since("now").unwrap();
        let now_after = Utc::now();

        assert!(now >= now_before && now <= now_after);

        let today = parse_since("today").unwrap();
        let expected_today = DateTime::<Utc>::from_naive_utc_and_offset(
            Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        );
        assert_eq!(today, expected_today);

        let yesterday = parse_since("yesterday").unwrap();
        let expected_yesterday = DateTime::<Utc>::from_naive_utc_and_offset(
            (Utc::now() - Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        );
        assert_eq!(yesterday, expected_yesterday);
    }

    #[test]
    fn test_invalid_inputs() {
        let cases = vec!["", "invalid", "2026/02/06"];

        for input in cases {
            assert!(parse_since(input).is_err(), "input should fail: {}", input);
        }
    }
}
