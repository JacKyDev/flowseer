use crate::github::GithubWorkflowClient;
use crate::github::{GithubClientBuilder, GithubClientError};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GithubWorkflowClientBuilder {
    base_builder: GithubClientBuilder,
    owner: Option<String>,
    repo: Option<String>,
    workflow_id: Option<String>,
    per_page: Option<u16>,
    page: Option<usize>,
    since: Option<String>,
}

impl Default for GithubWorkflowClientBuilder {
    fn default() -> Self {
        Self {
            base_builder: GithubClientBuilder::default(),
            owner: None,
            repo: None,
            workflow_id: None,
            per_page: Some(100),
            page: Some(1),
            since: None,
        }
    }
}

impl GithubWorkflowClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.base_builder = self.base_builder.token(token);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.base_builder = self.base_builder.timeout(timeout);
        self
    }

    pub fn retry(mut self, retry_enabled: bool) -> Self {
        self.base_builder = self.base_builder.retry(retry_enabled);
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.base_builder = self.base_builder.user_agent(user_agent);
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_builder = self.base_builder.base_url(base_url);
        self
    }

    pub fn ignore_trust(mut self) -> Self {
        self.base_builder = self.base_builder.ignore_trust();
        self
    }

    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn since(mut self, since: Option<impl Into<String>>) -> Self {
        self.since = since.map(|s| s.into());
        self
    }

    pub fn repo(mut self, repo: impl Into<String>) -> Self {
        self.repo = Some(repo.into());
        self
    }

    pub fn workflow_id(mut self, workflow_id: impl Into<String>) -> Self {
        self.workflow_id = Some(workflow_id.into());
        self
    }

    pub fn per_page(mut self, per_page: u16) -> Self {
        self.per_page = Some(per_page);
        self
    }

    pub fn build(self) -> Result<GithubWorkflowClient, GithubClientError> {
        let base_client = self.base_builder.build()?;

        let owner = self
            .owner
            .ok_or_else(|| GithubClientError::Config("Owner is required".to_string()))?;

        let repo = self
            .repo
            .ok_or_else(|| GithubClientError::Config("Repository name is required".to_string()))?;

        let workflow_id = self
            .workflow_id
            .ok_or_else(|| GithubClientError::Config("Workflow ID is required".to_string()))?;

        Ok(GithubWorkflowClient {
            client: base_client,
            owner,
            repo,
            workflow_id,
            per_page: self.per_page.unwrap_or(100),
            page: self.page.unwrap_or(1),
            since: self.since,
        })
    }
}
