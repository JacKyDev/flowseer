use chrono::Utc;
use reqwest::Response;
use std::time::Duration;
use tokio::time::sleep;

use crate::github::GithubClientError;

pub struct RateLimitHandler;

#[derive(Debug)]
pub enum RateLimitAction {
    Continue,
    WaitAndRetry(Duration),
    FailImmediately,
}

impl RateLimitHandler {
    pub fn check_rate_limit(
        response: &Response,
        current_attempt: usize,
        max_attempts: usize,
    ) -> RateLimitAction {
        if response.status() == 403 {
            if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
                if remaining.to_str().unwrap_or("1") == "0" {
                    if let Some(reset) = response.headers().get("x-ratelimit-reset") {
                        if let Ok(reset_str) = reset.to_str() {
                            if let Ok(reset_ts) = reset_str.parse::<i64>() {
                                let wait_seconds = reset_ts - Utc::now().timestamp();
                                if wait_seconds > 0 {
                                    return Self::decide_rate_limit_action(
                                        wait_seconds,
                                        current_attempt,
                                        max_attempts,
                                    );
                                }
                            }
                        }
                    }
                    return RateLimitAction::FailImmediately;
                }
            }
        }
        RateLimitAction::Continue
    }

    pub fn decide_rate_limit_action(
        wait_seconds: i64,
        current_attempt: usize,
        max_attempts: usize,
    ) -> RateLimitAction {
        if current_attempt >= max_attempts {
            eprintln!(
                "Github rate limit reached. No more retries available. API will reset in {} seconds.",
                wait_seconds
            );
            RateLimitAction::FailImmediately
        } else {
            eprintln!(
                "Github rate limit reached. Waiting {} seconds before retry {} of {}...",
                wait_seconds,
                current_attempt + 1,
                max_attempts
            );
            RateLimitAction::WaitAndRetry(Duration::from_secs(wait_seconds as u64 + 5))
        }
    }

    pub async fn wait_for_reset(duration: Duration) -> Result<(), GithubClientError> {
        sleep(duration).await;
        Ok(())
    }

    pub fn extract_rate_limit_info(response: &Response) -> RateLimitInfo {
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let reset = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let limit = response
            .headers()
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        RateLimitInfo {
            remaining,
            reset,
            limit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub reset: i64,
    pub limit: u32,
}

impl RateLimitInfo {
    pub fn seconds_until_reset(&self) -> i64 {
        self.reset - Utc::now().timestamp()
    }

    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    fn extract_rate_limit_info_from_headers(headers: &HeaderMap) -> RateLimitInfo {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        RateLimitInfo {
            remaining,
            reset,
            limit,
        }
    }

    #[test]
    fn test_rate_limit_info_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", HeaderValue::from_static("100"));
        headers.insert("x-ratelimit-reset", HeaderValue::from_static("1640995200"));
        headers.insert("x-ratelimit-limit", HeaderValue::from_static("5000"));

        let info = extract_rate_limit_info_from_headers(&headers);

        assert_eq!(info.remaining, 100);
        assert_eq!(info.reset, 1640995200);
        assert_eq!(info.limit, 5000);
    }

    #[test]
    fn test_rate_limit_info_with_missing_headers() {
        let headers = HeaderMap::new();
        let info = extract_rate_limit_info_from_headers(&headers);

        assert_eq!(info.remaining, 0);
        assert_eq!(info.reset, 0);
        assert_eq!(info.limit, 0);
    }

    #[test]
    fn test_rate_limit_info_seconds_until_reset() {
        let future_reset = Utc::now().timestamp() + 3600; // 1 hour from now
        let info = RateLimitInfo {
            remaining: 100,
            reset: future_reset,
            limit: 5000,
        };

        let seconds_until_reset = info.seconds_until_reset();
        assert!(seconds_until_reset > 3590 && seconds_until_reset <= 3600);
    }

    #[test]
    fn test_rate_limit_info_is_exhausted() {
        let exhausted_info = RateLimitInfo {
            remaining: 0,
            reset: Utc::now().timestamp() + 3600,
            limit: 5000,
        };

        let not_exhausted_info = RateLimitInfo {
            remaining: 100,
            reset: Utc::now().timestamp() + 3600,
            limit: 5000,
        };

        assert!(exhausted_info.is_exhausted());
        assert!(!not_exhausted_info.is_exhausted());
    }

    #[test]
    fn test_decide_rate_limit_action_no_retries_left() {
        let action = RateLimitHandler::decide_rate_limit_action(3600, 3, 3);
        match action {
            RateLimitAction::FailImmediately => {
                assert!(true);
            }
            _ => panic!("Should fail immediately when no retries are left"),
        }
    }

    #[test]
    fn test_decide_rate_limit_action_retries_available() {
        let action = RateLimitHandler::decide_rate_limit_action(3600, 1, 3);
        match action {
            RateLimitAction::WaitAndRetry(duration) => {
                assert_eq!(duration.as_secs(), 3605);
            }
            _ => panic!("Should wait and retry when attempts are still available"),
        }
    }

    #[test]
    fn test_decide_rate_limit_action_last_attempt() {
        let action = RateLimitHandler::decide_rate_limit_action(1800, 2, 3);
        match action {
            RateLimitAction::WaitAndRetry(duration) => {
                assert_eq!(duration.as_secs(), 1805);
            }
            _ => panic!("Should still attempt retry on last attempt"),
        }
    }
}
