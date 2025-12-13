use crate::github::{GithubClient, GithubClientError};

#[derive(Debug, Clone)]
pub struct GithubWorkflowClient {
    pub client: GithubClient,
    pub owner: String,
    pub repo: String,
    pub workflow_id: String,
    pub per_page: u16,
    pub page: usize,
}

impl GithubWorkflowClient {
    pub fn build_endpoint_url(&self) -> Result<String, GithubClientError> {
        self.client.build_url(format!(
            "/repos/{}/{}/actions/workflows/{}/runs?per_page={}&page={}",
            self.owner, self.repo, self.workflow_id, self.per_page, self.page
        ))
    }

    pub async fn get<T>(&self) -> Result<T, GithubClientError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.build_endpoint_url()?;
        let result = self.client.get::<T>(&url).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use crate::github::{GithubClientError, GithubWorkflowClientBuilder};
    use std::time::Duration;

    #[test]
    fn test_workflow_builder_pattern() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .repo("hello-world")
            .workflow_id("main.yml")
            .per_page(50)
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.owner, "dummy");
        assert_eq!(client.repo, "hello-world");
        assert_eq!(client.workflow_id, "main.yml");
        assert_eq!(client.per_page, 50);
    }

    #[test]
    fn test_workflow_builder_missing_owner() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .repo("hello-world")
            .workflow_id("main.yml")
            .build();

        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), GithubClientError::Config(_)));
    }

    #[test]
    fn test_workflow_builder_missing_repo() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .workflow_id("main.yml")
            .build();

        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), GithubClientError::Config(_)));
    }

    #[test]
    fn test_workflow_builder_missing_workflow_id() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .repo("hello-world")
            .build();

        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), GithubClientError::Config(_)));
    }

    #[test]
    fn test_workflow_url_generation() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .repo("hello-world")
            .workflow_id("main.yml")
            .per_page(100)
            .build()
            .unwrap();

        let expected_url = "https://api.github.com/repos/dummy/hello-world/actions/workflows/main.yml/runs?per_page=100&page=1";
        assert_eq!(client.build_endpoint_url().unwrap(), expected_url);
    }

    #[test]
    fn test_workflow_url_generation_with_page_change() {
        let mut client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .repo("hello-world")
            .workflow_id("main.yml")
            .per_page(100)
            .build()
            .unwrap();

        client.page = 5;

        let expected_url = "https://api.github.com/repos/dummy/hello-world/actions/workflows/main.yml/runs?per_page=100&page=5";
        assert_eq!(client.build_endpoint_url().unwrap(), expected_url);
    }

    #[test]
    fn test_default_per_page() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .owner("dummy")
            .repo("hello-world")
            .workflow_id("main.yml")
            .build()
            .unwrap();

        assert_eq!(client.per_page, 100);
    }

    #[test]
    fn test_builder_with_all_options() {
        let client = GithubWorkflowClientBuilder::new()
            .token("token123")
            .timeout(Duration::from_secs(60))
            .retry(false)
            .user_agent("MyApp/1.0")
            .owner("dummy")
            .repo("hello-world")
            .workflow_id("main.yml")
            .per_page(25)
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.per_page, 25);
    }
}
