//! Github Library
//!
//! A generic Github Libraryfor the Github API specific utitilies and logic.

pub use github_client::GithubClient;
pub use github_client_builder::GithubClientBuilder;
pub use github_workflow_client::GithubWorkflowClient;
pub use github_workflow_client_builder::GithubWorkflowClientBuilder;

pub use rate_limit_handler::{RateLimitAction, RateLimitHandler};

pub mod github_client;
pub mod github_client_builder;
pub mod github_workflow_client;
pub mod github_workflow_client_builder;
pub mod rate_limit_handler;
pub mod types;

pub use types::{
    GithubActorResponse, GithubClientError, GithubWorkflowRunResponse, GithubWorkflowRunsResponse,
};
