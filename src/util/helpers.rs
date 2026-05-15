use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub fn format_duration_from_str(start: Option<&str>, end: Option<&str>) -> String {
    match (start, end) {
        (Some(started), Some(ended)) => {
            match (
                started.parse::<DateTime<Utc>>(),
                ended.parse::<DateTime<Utc>>(),
            ) {
                (Ok(start_time), Ok(end_time)) => {
                    let duration = end_time - start_time;
                    let seconds = duration.num_seconds();

                    if seconds < 60 {
                        format!("{}s", seconds)
                    } else if seconds < 3600 {
                        format!("{}m {}s", seconds / 60, seconds % 60)
                    } else {
                        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
                    }
                }
                _ => "(invalid time)".to_string(),
            }
        }
        _ => "(no duration)".to_string(),
    }
}

pub fn mask_token(token: &str, display_prefix: Option<&str>) -> String {
    if let Some(prefix) = display_prefix {
        if token.starts_with(prefix) {
            return format!("{}**********", prefix);
        }
    }
    "**********".to_string()
}

pub fn wrap_at_spaces(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len() + 10);
    let mut current_line_len = 0;

    for word in input.split_whitespace() {
        if current_line_len + word.len() + 1 > limit && current_line_len > 0 {
            result.push('\n');
            current_line_len = 0;
        }

        if current_line_len > 0 {
            result.push(' ');
            current_line_len += 1;
        }

        result.push_str(word);
        current_line_len += word.len();
    }

    result
}

pub fn resolve_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_short_duration() {
        let start = "2025-08-02T12:00:00Z";
        let end = "2025-08-02T12:00:30Z";
        assert_eq!(format_duration_from_str(Some(start), Some(end)), "30s");
    }

    #[test]
    fn test_valid_medium_duration() {
        let start = "2025-08-02T12:00:00Z";
        let end = "2025-08-02T12:02:30Z";
        assert_eq!(format_duration_from_str(Some(start), Some(end)), "2m 30s");
    }

    #[test]
    fn test_valid_long_duration() {
        let start = "2025-08-02T10:00:00Z";
        let end = "2025-08-02T13:45:00Z";
        assert_eq!(format_duration_from_str(Some(start), Some(end)), "3h 45m");
    }

    #[test]
    fn test_invalid_datetime_format() {
        let start = "invalid";
        let end = "2025-08-02T12:00:00Z";
        assert_eq!(
            format_duration_from_str(Some(start), Some(end)),
            "(invalid time)"
        );
    }

    #[test]
    fn test_missing_inputs() {
        assert_eq!(
            format_duration_from_str(None, Some("2025-08-02T12:00:00Z")),
            "(no duration)"
        );
        assert_eq!(
            format_duration_from_str(Some("2025-08-02T12:00:00Z"), None),
            "(no duration)"
        );
        assert_eq!(format_duration_from_str(None, None), "(no duration)");
    }

    #[test]
    fn test_token_with_prefix() {
        let token = "ghp_1234567890abcdef";
        let prefix = Some("ghp_");
        assert_eq!(mask_token(token, prefix), "ghp_**********");
    }

    #[test]
    fn test_token_with_wrong_prefix() {
        let token = "abc_1234567890abcdef";
        let prefix = Some("ghp_");
        assert_eq!(mask_token(token, prefix), "**********");
    }

    #[test]
    fn test_token_without_prefix() {
        let token = "whatever_token";
        assert_eq!(mask_token(token, None), "**********");
    }

    #[test]
    fn test_wrap_simple() {
        let input = "This is a simple test string for wrapping.";
        let expected = "This is a\nsimple test\nstring for\nwrapping.";
        assert_eq!(wrap_at_spaces(input, 12), expected);
    }

    #[test]
    fn test_wrap_no_wrap_needed() {
        let input = "Short";
        assert_eq!(wrap_at_spaces(input, 10), "Short");
    }

    #[test]
    fn test_wrap_long_word() {
        let input = "supercalifragilisticexpialidocious is a word";
        let expected = "supercalifragilisticexpialidocious\nis a word";
        assert_eq!(wrap_at_spaces(input, 10), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(wrap_at_spaces("", 5), "");
    }
}
