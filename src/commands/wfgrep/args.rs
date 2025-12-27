use crate::commands::OutputFormat;
use clap::Parser;
use clap::value_parser;

#[derive(Parser, Debug)]
#[command(version, about = "Workflow Grep for Github workflow runs")]
pub struct WfGrepArgs {
    #[arg(
        short = 'p',
        long = "profile",
        help = "Configuration profile name (from ~/.flowseer)",
        long_help = "Name of the configuration profile to load from ~/.flowseer/<profile>.json. \
                     Values from the CLI override values from the profile."
    )]
    pub profile: Option<String>,

    #[arg(
        short = 'r',
        long = "gh-repo",
        help = "Repository name (e.g., 'my-project')",
        long_help = "Name of the Github repository to search in. Can be provided via CLI or configuration profile."
    )]
    pub repo: Option<String>,

    #[arg(
        short = 'w',
        long = "gh-workflow",
        help = "Workflow ID or filename (e.g., 'ci.yml', '123456')",
        long_help = "Github workflow identifier. Can be either the workflow filename (e.g., 'ci.yml', 'deploy.yaml') or the numeric workflow ID. Use 'gh workflow list' to see available workflows. Can be provided via CLI or configuration profile."
    )]
    pub workflow: Option<String>,

    #[arg(
        short = 'u',
        long = "gh-owner",
        help = "Organization/user name (Github owner)",
        long_help = "Github organization or username that owns the repository. This is the first part of the full repository path (owner/repo). Can be provided via CLI or configuration profile."
    )]
    pub owner: Option<String>,

    #[arg(
        short = 't',
        long = "gh-token",
        env = "GITHUB_TOKEN",
        help = "Github token (CLI arg or GITHUB_TOKEN env var, or profile)",
        long_help = "Github personal access token for API authentication. Can be provided via --gh-token, GITHUB_TOKEN environment variable, or configuration profile. Token needs 'repo' and 'actions:read' permissions."
    )]
    pub token: Option<String>,

    #[arg(
        value_enum,
        short,
        long,
        default_value_t = OutputFormat::Table,
        help = "Output format: table or json",
        long_help = "Format for displaying results. 'table' provides a human-readable tabular output, while 'json' outputs structured data suitable for further processing or scripting."    
    )]
    pub output: OutputFormat,

    #[arg(
        short,
        long,
        help = "Filter by name containing terms",
        long_help = "Filter workflow runs by names containing these terms. Multiple terms can be provided and all must match (AND logic). Case-insensitive matching is used."
    )]
    pub contains: Vec<String>,

    #[arg(
        long,
        default_value = "10",
        value_parser = value_parser!(u8).range(1..=50),
        help = "Number of concurrent API requests",
        long_help = "Number of concurrent requests to make to the Github API. Higher values speed up data retrieval but may hit rate limits. Adjust based on your API quota and network capacity."
    )]
    pub concurrency: u8,

    #[arg(
        long,
        default_value = "30",
        help = "Request timeout in seconds",
        long_help = "Timeout for individual API requests in seconds. Increase if you experience timeout errors with slow connections or large responses."
    )]
    pub timeout: u64,

    #[arg(
        long,
        help = "Retry failed requests automatically",
        long_help = "Automatically retry failed API requests. Useful for handling transient network errors or temporary Github API issues. Failed requests will be retried with exponential backoff."
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
