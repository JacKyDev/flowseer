//! # Flowseer `wfgrep` CLI Arguments
//!
//! This module defines the command-line interface (CLI) arguments for `flowseer wfgrep`.
//!
//! The `wfgrep` command allows querying and filtering GitHub workflow runs for a repository.
//! It supports fetching data via the GitHub API with optional limits, filtering, and sorting.
//!
//! CLI arguments can be provided directly, or values can be loaded from a configuration
//! profile located at `~/.flowseer/<profile>.json`. Values provided via CLI always override
//! profile values when specified.

use crate::{
    commands::{OutputFormat, SortOrder, types::Sort},
    util::parse_since,
};
use chrono::{DateTime, Utc};
use clap::{Parser, value_parser};

/// CLI arguments for the `flowseer wfgrep` command.
///
/// This struct represents all user-configurable options for fetching and processing
/// GitHub workflow runs, including repository selection, workflow identifiers, API
/// authentication, limits, filters, output formatting, concurrency, and retry behavior.
///
/// Values can be specified directly on the CLI, or loaded from a configuration profile
/// (`~/.flowseer/<profile>.json`). CLI arguments always override profile values when set.
///
/// Example usage:
///
/// ```text
/// wfgrep --gh-owner myorg --gh-repo myrepo --gh-workflow ci.yml --gh-token ghp_xxx
/// ```
///
/// For detailed argument descriptions, see each field's `help` and `long_help`.
#[derive(Parser, Debug)]
#[command(version, about = "Workflow Grep for Github workflow runs")]
pub struct WfGrepArgs {
    /// Configuration profile name
    ///
    /// Load default values from `~/.flowseer/<profile>.json`. CLI arguments override profile values.
    #[arg(
        short = 'p',
        long = "profile",
        help = "Configuration profile name (from ~/.flowseer)",
        long_help = "Name of the configuration profile to load from ~/.flowseer/<profile>.json. \
                     Values from the CLI override values from the profile."
    )]
    pub profile: Option<String>,

    /// Repository name
    ///
    /// The GitHub repository to search for workflow runs.
    #[arg(
        short = 'r',
        long = "gh-repo",
        help = "Repository name (e.g., 'my-project')",
        long_help = "Name of the GitHub repository to search in. CLI overrides profile values."
    )]
    pub repo: Option<String>,

    /// Workflow ID or filename
    ///
    /// Can be a numeric workflow ID or workflow YAML filename.
    #[arg(
        short = 'w',
        long = "gh-workflow",
        help = "Workflow ID or filename (e.g., 'ci.yml', '123456')",
        long_help = "GitHub workflow identifier. Either workflow filename (e.g., 'ci.yml', 'deploy.yaml') or numeric workflow ID. Use 'gh workflow list' to see available workflows. CLI overrides profile values."
    )]
    pub workflow: Option<String>,

    /// GitHub organization or username
    ///
    /// The owner of the repository.
    #[arg(
        short = 'u',
        long = "gh-owner",
        help = "Organization/user name (GitHub owner)",
        long_help = "GitHub organization or username that owns the repository. This is the first part of the full repository path (owner/repo). CLI overrides profile values."
    )]
    pub owner: Option<String>,

    /// Maximum number of workflow runs fetched from API
    ///
    /// Limits API requests and reduces network traffic.
    #[arg(
        short = 'l',
        long = "limit",
        value_parser = value_parser!(u16).range(1..=65_535),
        help = "Maximum number of workflow runs to fetch from GitHub",
        long_help = "Limits the number of workflow runs fetched from the GitHub API. Only as many pages as needed are fetched (max 100 per page). Does not affect final output display. \nCLI arguments override profile values if set. Leaving the CLI value empty does not remove the profile value; it must be removed from the profile to have no value."
    )]
    pub limit: Option<u16>,

    /// Maximum number of workflow runs displayed after processing
    ///
    /// Works after filtering, sorting, and aggregation.
    #[arg(
        short = 'H',
        long = "head",
        help = "Limits the number of workflow runs displayed after processing",
        long_help = "Limits the number of workflow runs displayed after all processing steps (filtering, sorting, aggregation) have been applied. Acts like Unix 'head'. Unlike --limit, this only affects final output and does not reduce API requests. \nCLI arguments override profile values if set. Leaving the CLI value empty does not remove the profile value; it must be removed from the profile to have no value."
    )]
    pub head: Option<u16>,

    /// GitHub token
    ///
    /// Personal access token for API authentication.
    #[arg(
        short = 't',
        long = "gh-token",
        env = "GITHUB_TOKEN",
        help = "GitHub token (CLI arg or GITHUB_TOKEN env var, or profile)",
        long_help = "GitHub personal access token for API authentication. Can be provided via --gh-token, GITHUB_TOKEN environment variable, or configuration profile. Token needs 'repo' and 'actions:read' permissions. CLI overrides profile values."
    )]
    pub token: Option<String>,

    /// Output format
    ///
    /// Table or JSON.
    #[arg(
        value_enum,
        short,
        long,
        default_value_t = OutputFormat::Table,
        help = "Output format",
        long_help = "Format for displaying results. 'table' provides a human-readable tabular output, 'json' provides structured data. CLI overrides profile values."
    )]
    pub output: OutputFormat,

    /// Filter workflow runs by name containing terms
    ///
    /// Multiple terms are ANDed together.
    #[arg(
        short,
        long,
        help = "Filter by name containing terms",
        long_help = "Filter workflow runs by names containing these terms. Multiple terms are combined with AND logic. Case-insensitive. CLI overrides profile values."
    )]
    pub contains: Vec<String>,

    /// Only include workflow runs created after the specified time.
    ///
    /// Supported input formats:
    ///
    /// - **RFC3339 / ISO-8601 timestamp**
    ///   - `2026-02-06T12:00:00Z`
    ///   - `2026-02-06T12:00:00+01:00`
    ///
    /// - **Date only**
    ///   - `2026-02-06` (interpreted as `2026-02-06T00:00:00Z`)
    ///
    /// - **Relative time**
    ///   - `7d`  → 7 days ago
    ///   - `24h` → 24 hours ago
    ///   - `30m` → 30 minutes ago
    ///
    /// - **Keywords**
    ///   - `yesterday`
    ///   - `today`
    ///   - `now`
    ///
    /// # Examples
    ///
    /// ```bash
    /// flowseer wfgrep --since 2026-02-06
    /// flowseer wfgrep --since 2026-02-06T12:00:00Z
    /// flowseer wfgrep --since 7d
    /// flowseer wfgrep --since yesterday
    /// ```
    #[arg(
        long,
        value_parser = parse_since,
        help = "Only include workflow runs created after this date (ISO-8601)",
        long_help = "Accepts YYYY-MM-DD, RFC3339 timestamps, or relative times like 7d, 24h, yesterday."
    )]
    pub since: Option<DateTime<Utc>>,

    /// Sort workflow runs by field
    ///
    /// Determines which workflow run field is used for sorting the final result set.
    /// Sorting is applied after filtering and before `--head`.
    #[arg(
        long,
        value_enum,
        default_value_t = Sort::CreatedAt,
        help = "Field to sort workflow runs by",
        long_help = "Defines the field used to sort workflow runs in the final result set. \
                     Sorting is applied after filtering and before applying --head. \
                     Currently, only 'created-at' is supported. \
                     CLI arguments override profile values if set."
    )]
    pub sort: Sort,

    /// Sort order
    ///
    /// Controls whether the selected sort field is applied in ascending or descending order.
    #[arg(
        long = "sort-order",
        value_enum,
        default_value_t = SortOrder::Desc,
        help = "Sort order (ascending or descending)",
        long_help = "Defines the order in which the selected sort field is applied. \
                     'desc' sorts from newest to oldest, 'asc' from oldest to newest. \
                     The sort order always applies to the field specified by --sort. \
                     Defaults to descending. \
                     CLI arguments override profile values if set."
    )]
    pub sort_order: SortOrder,

    /// Number of concurrent API requests
    ///
    /// Increasing concurrency may speed up fetching but risks rate limits.
    #[arg(
        long,
        default_value = "10",
        value_parser = value_parser!(u8).range(1..=50),
        help = "Number of concurrent API requests",
        long_help = "Number of concurrent requests to make to the GitHub API. Higher values speed up data retrieval but may hit rate limits. \nCLI arguments override profile values if set. Leaving the CLI value empty does not remove the profile value; it must be removed from the profile to have no value."
    )]
    pub concurrency: u8,

    /// API request timeout in seconds
    ///
    /// Increase if requests fail due to network latency.    
    #[arg(
        long,
        default_value = "30",
        help = "Request timeout in seconds",
        long_help = "Timeout for individual API requests in seconds. Increase if experiencing timeout errors. \nCLI arguments override profile values if set. Leaving the CLI value empty does not remove the profile value; it must be removed from the profile to have no value."
    )]
    pub timeout: u64,

    /// Retry failed API requests automatically
    ///
    /// Uses exponential backoff.
    #[arg(
        long,
        help = "Retry failed requests automatically",
        long_help = "Automatically retry failed API requests. Useful for transient network errors or temporary GitHub API issues. \nCLI arguments override profile values if set. Leaving the CLI value empty does not remove the profile value; it must be removed from the profile to have no value."
    )]
    pub retry: bool,

    #[arg(long, hide = true)]
    pub dev_mode: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_minimal_args() {
        let args = WfGrepArgs::try_parse_from([
            "wfgrep",
            "--gh-repo",
            "my-repo",
            "--gh-workflow",
            "ci.yml",
            "--gh-owner",
            "myorg",
            "--gh-token",
            "ghp_test123",
        ])
        .unwrap();

        assert_eq!(args.repo.as_deref(), Some("my-repo"));
        assert_eq!(args.workflow.as_deref(), Some("ci.yml"));
        assert_eq!(args.owner.as_deref(), Some("myorg"));
        assert_eq!(args.token.as_deref(), Some("ghp_test123"));
        assert_eq!(args.output, OutputFormat::Table);
        assert_eq!(args.concurrency, 10);
        assert_eq!(args.timeout, 30);
        assert!(!args.retry);
    }

    #[test]
    fn test_parse_all_args() {
        let args = WfGrepArgs::try_parse_from([
            "wfgrep",
            "--gh-repo",
            "my-repo",
            "--gh-workflow",
            "deploy.yml",
            "--gh-owner",
            "myorg",
            "--gh-token",
            "ghp_test123",
            "--output",
            "json",
            "--contains",
            "fix",
            "--contains",
            "bug",
            "--concurrency",
            "5",
            "--timeout",
            "60",
            "--retry",
        ])
        .unwrap();

        assert_eq!(args.repo.as_deref(), Some("my-repo"));
        assert_eq!(args.workflow.as_deref(), Some("deploy.yml"));
        assert_eq!(args.owner.as_deref(), Some("myorg"));
        assert_eq!(args.token.as_deref(), Some("ghp_test123"));
        assert_eq!(args.output, OutputFormat::Json);
        assert_eq!(args.contains, vec!["fix", "bug"]);
        assert_eq!(args.concurrency, 5);
        assert_eq!(args.timeout, 60);
        assert!(args.retry);
    }

    #[test]
    fn test_short_flags() {
        let args = WfGrepArgs::try_parse_from([
            "wfgrep",
            "-r",
            "my-repo",
            "-w",
            "ci.yml",
            "-u",
            "myorg",
            "-t",
            "ghp_test123",
        ])
        .unwrap();

        assert_eq!(args.repo.as_deref(), Some("my-repo"));
        assert_eq!(args.workflow.as_deref(), Some("ci.yml"));
        assert_eq!(args.owner.as_deref(), Some("myorg"));
        assert_eq!(args.token.as_deref(), Some("ghp_test123"));
    }

    #[test]
    fn test_missing_required_args() {
        let result = WfGrepArgs::try_parse_from([
            "wfgrep", "--repo", "my-repo", // Missing workflow, owner, token
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_output_format() {
        let result = WfGrepArgs::try_parse_from([
            "wfgrep",
            "--repo",
            "my-repo",
            "--workflow",
            "ci.yml",
            "--gh-owner",
            "myorg",
            "--token",
            "ghp_test123",
            "--output",
            "xml", // invalid format
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_contains_filters() {
        let args = WfGrepArgs::try_parse_from([
            "wfgrep",
            "--gh-repo",
            "my-repo",
            "--gh-workflow",
            "ci.yml",
            "--gh-owner",
            "myorg",
            "--gh-token",
            "ghp_test123",
            "--contains",
            "fix",
            "--contains",
            "bug",
            "--contains",
            "urgent",
        ])
        .unwrap();

        assert_eq!(args.contains, vec!["fix", "bug", "urgent"]);
    }

    #[test]
    fn test_default_values() {
        let args = WfGrepArgs::try_parse_from([
            "wfgrep",
            "--gh-repo",
            "my-repo",
            "--gh-workflow",
            "ci.yml",
            "--gh-owner",
            "myorg",
            "--gh-token",
            "ghp_test123",
        ])
        .unwrap();

        assert_eq!(args.output, OutputFormat::Table);
        assert_eq!(args.concurrency, 10);
        assert_eq!(args.timeout, 30);
        assert!(!args.retry);
        assert!(args.contains.is_empty());
    }
}
