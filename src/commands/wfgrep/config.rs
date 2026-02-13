use crate::commands::WorkflowMetaData;
use crate::commands::WorkflowTableRowData;
use crate::commands::wfgrep::WfGrepArgs;
use crate::commands::wfgrep::types::EffectiveConfig;
use crate::github::GithubWorkflowRunResponse;
use crate::profile::Profile;
use crate::util::format_duration_from_str;
use crate::util::mask_token;
use crate::util::pick::pick_vec;
use crate::util::pick_bool;
use crate::util::pick_default;
use crate::util::pick_optional;
use crate::util::pick_required;
use crate::util::wrap_at_spaces;

impl EffectiveConfig {
    pub fn from_args_and_profile(
        args: WfGrepArgs,
        profile: Option<Profile>,
    ) -> anyhow::Result<Self> {
        let profile = profile.unwrap_or_default();

        let repo = pick_required(args.repo, profile.repo, "repo")?;
        let owner = pick_required(args.owner, profile.owner, "owner")?;
        let workflow = pick_required(args.workflow, profile.workflow, "workflow")?;
        let token = pick_required(args.token, profile.token, "token")?;

        let contains = pick_vec(args.contains, profile.contains);

        let limit = pick_optional(args.limit, profile.limit);
        let head = pick_optional(args.head, profile.head);

        let output = args.output;

        let sort = pick_default(args.sort, profile.sort);
        let sort_order = pick_default(args.sort_order, profile.sort_order);

        let concurrency = pick_default(args.concurrency, profile.concurrency);
        let timeout = pick_default(args.timeout, profile.timeout);
        let retry = pick_bool(args.retry, profile.retry);

        let dev_mode = args.dev_mode;

        Ok(Self {
            repo,
            owner,
            workflow,
            token,
            output,
            contains,
            limit,
            head,
            concurrency,
            timeout,
            retry,
            dev_mode,
            sort,
            sort_order,
        })
    }
}

impl From<&GithubWorkflowRunResponse> for WorkflowTableRowData {
    fn from(run: &GithubWorkflowRunResponse) -> Self {
        WorkflowTableRowData {
            user: run
                .actor
                .as_ref()
                .map_or("(no user)", |a| &a.login)
                .to_string(),
            conclusion: run
                .conclusion
                .as_deref()
                .unwrap_or("(no conclusion)")
                .to_string(),
            duration: format_duration_from_str(
                run.run_started_at.as_deref(),
                Some(&run.updated_at),
            ),
            name: wrap_at_spaces(run.name.as_deref().unwrap_or("unknown"), 30),
            id: run.id,
            start_date: run
                .run_started_at
                .as_deref()
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

impl From<&EffectiveConfig> for WorkflowMetaData {
    fn from(args: &EffectiveConfig) -> Self {
        WorkflowMetaData {
            contains: args.contains.clone(),
            output: args.output,
            owner: args.owner.clone(),
            repo: args.repo.clone(),
            workflow: args.workflow.clone(),
            limit: args.limit,
            head: args.head,
            sort: args.sort,
            sort_order: args.sort_order,
            token: mask_token(&args.token, Some("ghp_")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::OutputFormat;
    use crate::commands::Sort;
    use crate::commands::SortOrder;
    use crate::commands::wfgrep::types::EffectiveConfig;
    use crate::profile::Profile;

    fn empty_args() -> WfGrepArgs {
        WfGrepArgs {
            repo: None,
            owner: None,
            workflow: None,
            token: None,
            limit: None,
            head: None,
            output: OutputFormat::Table,
            contains: vec![].into(),
            concurrency: 10,
            timeout: 30,
            retry: false,
            dev_mode: false,
            profile: None,
            sort: Sort::CreatedAt,
            sort_order: SortOrder::Asc,
        }
    }

    fn full_profile() -> Profile {
        Profile {
            repo: Some("profile-repo".into()),
            owner: Some("profile-owner".into()),
            workflow: Some("profile-workflow".into()),
            token: Some("profile-token".into()),
            limit: None,
            head: None,
            output: None,
            contains: vec![].into(),
            concurrency: None,
            timeout: None,
            retry: None,
            sort: None,
            sort_order: None,
        }
    }

    #[test]
    fn cli_overrides_profile() {
        let args = WfGrepArgs {
            repo: Some("cli-repo".into()),
            owner: Some("cli-owner".into()),
            workflow: Some("cli-workflow".into()),
            token: Some("cli-token".into()),
            ..empty_args()
        };

        let profile = full_profile();

        let cfg = EffectiveConfig::from_args_and_profile(args, Some(profile)).unwrap();

        assert_eq!(cfg.repo, "cli-repo");
        assert_eq!(cfg.owner, "cli-owner");
        assert_eq!(cfg.workflow, "cli-workflow");
        assert_eq!(cfg.token, "cli-token");
    }

    #[test]
    fn cli_empty_profile_used() {
        let args = empty_args();
        let profile = full_profile();

        let cfg = EffectiveConfig::from_args_and_profile(args, Some(profile)).unwrap();

        assert_eq!(cfg.repo, "profile-repo");
        assert_eq!(cfg.owner, "profile-owner");
        assert_eq!(cfg.workflow, "profile-workflow");
        assert_eq!(cfg.token, "profile-token");
    }

    #[test]
    fn cli_and_profile_combined() {
        let args = WfGrepArgs {
            repo: Some("cli-repo".into()),
            owner: None,
            workflow: None,
            token: Some("cli-token".into()),
            ..empty_args()
        };

        let profile = Profile {
            repo: None,
            owner: Some("profile-owner".into()),
            workflow: Some("profile-workflow".into()),
            token: None,
            output: None,
            limit: None,
            head: None,
            contains: vec![].into(),
            concurrency: None,
            timeout: None,
            retry: None,
            sort: None,
            sort_order: None,
        };

        let cfg = EffectiveConfig::from_args_and_profile(args, Some(profile)).unwrap();

        assert_eq!(cfg.repo, "cli-repo");
        assert_eq!(cfg.owner, "profile-owner");
        assert_eq!(cfg.workflow, "profile-workflow");
        assert_eq!(cfg.token, "cli-token");
    }

    #[test]
    fn error_if_required_fields_missing() {
        let args = empty_args();
        let profile = Profile {
            repo: None,
            owner: None,
            workflow: None,
            token: None,
            output: None,
            limit: None,
            head: None,
            contains: vec![].into(),
            concurrency: None,
            timeout: None,
            retry: None,
            sort: None,
            sort_order: None,
        };

        let err = EffectiveConfig::from_args_and_profile(args, Some(profile));
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("repo")
                || msg.contains("owner")
                || msg.contains("workflow")
                || msg.contains("token")
        );
    }
}
