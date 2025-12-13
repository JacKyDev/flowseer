use crate::github::{GithubClientError, RateLimitAction, RateLimitHandler};
use anyhow::Context;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::time::sleep;

const MAX_RETRY_ATTEMPTS: usize = 3;
const RETRY_DELAY_BASE_MS: u64 = 1000;

#[derive(Clone, Debug)]
pub struct GithubClient {
    pub client: reqwest::Client,
    pub retry_enabled: bool,
    pub base_url: String,
    pub ignore_trust: bool,
}

impl GithubClient {
    pub(crate) fn new(
        token: String,
        timeout: Duration,
        retry_enabled: bool,
        user_agent: String,
        base_url: Option<String>,
        ignore_trust: bool,
    ) -> Result<Self, GithubClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&user_agent)
                .map_err(|_| GithubClientError::Config("Invalid user agent format".to_string()))?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| GithubClientError::Config("Invalid token format".to_string()))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(ignore_trust)
            .timeout(timeout)
            .build()
            .context("Failed to create HTTP client")
            .map_err(|e| GithubClientError::Config(e.to_string()))?;

        Ok(Self {
            client,
            retry_enabled,
            base_url: base_url.unwrap_or_else(|| "https://api.github.com".to_string()),
            ignore_trust,
        })
    }

    pub async fn get<T>(&self, url: &str) -> Result<T, GithubClientError>
    where
        T: DeserializeOwned,
    {
        let max_attempts = if self.retry_enabled {
            MAX_RETRY_ATTEMPTS
        } else {
            1
        };

        for attempt in 1..=max_attempts {
            match self.client.get(url).send().await {
                Ok(response) => {
                    match RateLimitHandler::check_rate_limit(&response, attempt, max_attempts) {
                        RateLimitAction::Continue => {}
                        RateLimitAction::WaitAndRetry(duration) => {
                            RateLimitHandler::wait_for_reset(duration).await?;
                            continue;
                        }
                        RateLimitAction::FailImmediately => {
                            return Err(GithubClientError::RateLimit);
                        }
                    }

                    if response.status().is_success() {
                        match response.json::<T>().await {
                            Ok(data) => return Ok(data),
                            Err(e) if attempt < max_attempts => {
                                eprintln!("JSON parse error on attempt {}: {}", attempt, e);
                                sleep(Duration::from_millis(RETRY_DELAY_BASE_MS * attempt as u64))
                                    .await;
                                continue;
                            }
                            Err(e) => return Err(GithubClientError::Network(e)),
                        }
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        let error_msg = format!("HTTP {}: {}", status, error_text);

                        if attempt < max_attempts && status.is_server_error() {
                            eprintln!("Server error on attempt {}: {}", attempt, error_msg);
                            sleep(Duration::from_millis(RETRY_DELAY_BASE_MS * attempt as u64))
                                .await;
                            continue;
                        }

                        return Err(GithubClientError::GithubApi(error_msg));
                    }
                }
                Err(e) if attempt < max_attempts => {
                    eprintln!("Network error on attempt {}: {}", attempt, e);
                    sleep(Duration::from_millis(RETRY_DELAY_BASE_MS * attempt as u64)).await;
                    continue;
                }
                Err(e) => return Err(GithubClientError::Network(e)),
            }
        }

        unreachable!("Loop should always return before reaching this point")
    }

    pub fn build_url<S: AsRef<str>>(&self, endpoint: S) -> Result<String, GithubClientError> {
        let endpoint = endpoint.as_ref();

        if endpoint.starts_with("http://") {
            Err(GithubClientError::InsecureProtocol)
        } else if endpoint.starts_with("https://") {
            if self.base_url == endpoint || endpoint.starts_with(&(self.base_url.clone() + "/")) {
                Ok(endpoint.to_string())
            } else {
                Err(GithubClientError::UntrustedDomain)
            }
        } else if endpoint.starts_with("/") {
            Ok(format!("{}{}", &self.base_url, endpoint))
        } else {
            Ok(format!("{}/{}", &self.base_url, endpoint))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::GithubClientBuilder;

    fn create_client() -> GithubClient {
        GithubClientBuilder::new()
            .token("test_token")
            .timeout(Duration::from_secs(30))
            .retry(false)
            .build()
            .expect("Failed to create test client")
    }

    #[test]
    fn test_builder_pattern() {
        let client = GithubClientBuilder::new()
            .token("token123")
            .timeout(Duration::from_secs(60))
            .retry(false)
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_missing_token() {
        let client = GithubClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .build();

        assert!(client.is_err());
        assert!(matches!(client.unwrap_err().kind(), "Config"));
    }

    #[test]
    fn test_build_url() {
        let client = GithubClientBuilder::new().token("token").build().unwrap();

        // Absolute URL
        assert_eq!(
            client
                .build_url("https://api.github.com/repos/owner/repo")
                .unwrap(),
            "https://api.github.com/repos/owner/repo"
        );

        // Relative URL with leading slash
        assert_eq!(
            client.build_url("/repos/owner/repo").unwrap(),
            "https://api.github.com/repos/owner/repo"
        );

        // Relative URL without leading slash
        assert_eq!(
            client.build_url("repos/owner/repo").unwrap(),
            "https://api.github.com/repos/owner/repo"
        );
    }

    #[test]
    fn test_custom_user_agent() {
        let client = GithubClientBuilder::new()
            .token("token")
            .user_agent("MyApp/1.0")
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_relative_path_with_leading_slash() {
        let client = create_client();
        let result = client.build_url("/users/dummy");
        assert_eq!(result.unwrap(), "https://api.github.com/users/dummy");
    }

    #[test]
    fn test_relative_path_without_leading_slash() {
        let client = create_client();
        let result = client.build_url("users/dummy");
        assert_eq!(result.unwrap(), "https://api.github.com/users/dummy");
    }

    #[test]
    fn test_empty_endpoint() {
        let client = create_client();
        let result = client.build_url("");
        assert_eq!(result.unwrap(), "https://api.github.com/");
    }

    #[test]
    fn test_valid_https_github_api_url() {
        let client = create_client();
        let result = client.build_url("https://api.github.com/users/dummy");
        assert_eq!(result.unwrap(), "https://api.github.com/users/dummy");
    }

    #[test]
    fn test_valid_https_github_api_base_url() {
        let client = create_client();
        let result = client.build_url("https://api.github.com");
        assert_eq!(result.unwrap(), "https://api.github.com");
    }

    #[test]
    fn test_http_url_rejected() {
        let client = create_client();
        let result = client.build_url("http://api.github.com/users/dummy");
        assert_eq!(result.unwrap_err().kind(), "InsecureProtocol");
    }

    #[test]
    fn test_http_non_github_url_rejected() {
        let client = create_client();
        let result = client.build_url("http://malicious-site.com");
        assert_eq!(result.unwrap_err().kind(), "InsecureProtocol");
    }

    #[test]
    fn test_https_non_github_url_rejected() {
        let client = create_client();
        let result = client.build_url("https://malicious-site.com");
        assert_eq!(result.unwrap_err().kind(), "UntrustedDomain");
    }

    #[test]
    fn test_https_github_subdomain_rejected() {
        let client = create_client();
        let result = client.build_url("https://evil.api.github.com/users");
        assert_eq!(result.unwrap_err().kind(), "UntrustedDomain");
    }

    #[test]
    fn test_https_github_wrong_subdomain_rejected() {
        let client = create_client();
        let result = client.build_url("https://github.com/users");
        assert_eq!(result.unwrap_err().kind(), "UntrustedDomain");
    }

    #[test]
    fn test_complex_relative_paths() {
        let client = create_client();

        // Mit Query Parameters
        let result = client.build_url("search/repositories?q=rust");
        assert_eq!(
            result.unwrap(),
            "https://api.github.com/search/repositories?q=rust"
        );

        // Mit Fragment
        let result = client.build_url("users/dummy#profile");
        assert_eq!(
            result.unwrap(),
            "https://api.github.com/users/dummy#profile"
        );
    }

    #[test]
    fn test_edge_cases() {
        let client = create_client();

        // Mehrere Slashes
        let result = client.build_url("//users/dummy");
        assert_eq!(result.unwrap(), "https://api.github.com//users/dummy");

        // Nur Slash
        let result = client.build_url("/");
        assert_eq!(result.unwrap(), "https://api.github.com/");
    }

    #[test]
    fn test_string_types() {
        let client = create_client();

        // &str
        let result = client.build_url("/users/dummy");
        assert!(result.is_ok());

        // String
        let endpoint = String::from("users/dummy");
        let result = client.build_url(endpoint);
        assert!(result.is_ok());

        // &String
        let endpoint = String::from("users/dummy");
        let result = client.build_url(&endpoint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", GithubClientError::InsecureProtocol),
            "HTTP protocol is not allowed, use HTTPS"
        );
        assert_eq!(
            format!("{}", GithubClientError::UntrustedDomain),
            "URL must be from api.github.com domain"
        );
        assert_eq!(
            format!("{}", GithubClientError::InvalidUrl),
            "Invalid URL format"
        );
    }

    #[test]
    fn test_potential_bypass_attempts() {
        let client = create_client();

        // Versuche, die Validierung zu umgehen
        let malicious_urls = vec![
            "https://api.github.com.evil.com/users",
            "https://api.github.com@evil.com/users",
            "https://evil.com/api.github.com/users",
            "https://api-github-com.evil.com/users",
        ];

        for url in malicious_urls {
            let result = client.build_url(url);
            assert_eq!(
                result.unwrap_err().kind(),
                "UntrustedDomain",
                "Should reject malicious URL: {}",
                url
            );
        }
    }
}
