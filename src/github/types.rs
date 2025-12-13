use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubActorResponse {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowRunsResponse {
    pub workflow_runs: Vec<GithubWorkflowRunResponse>,
    pub total_count: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubWorkflowRunResponse {
    pub id: u64,
    pub name: Option<String>,
    pub head_branch: Option<String>,
    pub event: Option<String>,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub actor: Option<GithubActorResponse>,
    pub run_started_at: Option<String>,

    path: Option<String>,
    workflow_id: Option<u64>,
    html_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GithubClientError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Github API error: {0}")]
    GithubApi(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("HTTP protocol is not allowed, use HTTPS")]
    InsecureProtocol,
    #[error("URL must be from api.github.com domain")]
    UntrustedDomain,
    #[error("Invalid URL format")]
    InvalidUrl,
}

impl GithubClientError {
    pub fn kind(&self) -> &'static str {
        match self {
            GithubClientError::Config(_) => "Config",
            GithubClientError::Network(_) => "Network",
            GithubClientError::GithubApi(_) => "GithubApi",
            GithubClientError::RateLimit => "RateLimit",
            GithubClientError::InsecureProtocol => "InsecureProtocol",
            GithubClientError::UntrustedDomain => "UntrustedDomain",
            GithubClientError::InvalidUrl => "InvalidUrl",
        }
    }
}
