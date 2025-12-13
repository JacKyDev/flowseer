use std::time::Duration;

use crate::github::{GithubClient, GithubClientError};

#[derive(Debug, Clone)]
pub struct GithubClientBuilder {
    token: Option<String>,
    timeout: Duration,
    retry_enabled: bool,
    user_agent: Option<String>,
    base_url: Option<String>,
    ignore_trust: bool,
}

impl Default for GithubClientBuilder {
    fn default() -> Self {
        Self {
            token: None,
            timeout: Duration::from_secs(30),
            retry_enabled: true,
            user_agent: None,
            base_url: Some("https://api.github.com".to_string()),
            ignore_trust: false,
        }
    }
}

impl GithubClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn ignore_trust(mut self) -> Self {
        self.ignore_trust = true;
        self
    }

    pub fn retry(mut self, retry_enabled: bool) -> Self {
        self.retry_enabled = retry_enabled;
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn build(self) -> Result<GithubClient, GithubClientError> {
        let token = self
            .token
            .ok_or_else(|| GithubClientError::Config("Token is required".to_string()))?;

        let user_agent = self
            .user_agent
            .unwrap_or_else(|| format!("flowseer/github-client-lib/{}", env!("CARGO_PKG_VERSION")));

        GithubClient::new(
            token,
            self.timeout,
            self.retry_enabled,
            user_agent,
            self.base_url,
            self.ignore_trust,
        )
    }
}
