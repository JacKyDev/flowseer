use crate::AppMetadata;
use crate::commands::Command;
use crate::commands::wfgrep::WfGrepCommand;
use crate::commands::wfgrep::apply_user_sort;
use crate::commands::wfgrep::calculate_pagination;
use crate::commands::wfgrep::filter_runs;
use crate::commands::wfgrep::handle_paginated_requests;
use crate::commands::wfgrep::output_data;
use crate::commands::wfgrep::sort_runs;
use crate::commands::wfgrep::types::EffectiveConfig;
use crate::commands::wfgrep::{ErrorType, PaginatedError};
use crate::github::{
    GithubWorkflowClientBuilder, GithubWorkflowRunResponse, GithubWorkflowRunsResponse,
};
use anyhow::Result;
use async_trait::async_trait;
use tokio::time::Duration;

#[cfg(not(debug_assertions))]
use crate::commands::wfgrep::{print_dev_mode_impacts, print_dev_mode_warning};

#[async_trait]
impl Command for WfGrepCommand {
    async fn run(&self) -> Result<()> {
        let EffectiveConfig {
            ref token,
            ref owner,
            ref repo,
            limit,
            head,
            sort,
            sort_order,
            timeout,
            retry,
            ref workflow,
            ref concurrency,
            ref since,
            dev_mode,
            ..
        } = self.args;

        if dev_mode {
            let suppress = std::env::var("SUPPRESS_DEV_WARNING").unwrap_or_default() == "1";
            if !suppress {
                #[cfg(not(debug_assertions))]
                {
                    print_dev_mode_warning();
                    print_dev_mode_impacts();
                }
            }
        }

        let mut errors = Vec::new();

        let per_page = match limit {
            Some(l) => {
                if l > 100 {
                    100
                } else {
                    l
                }
            }
            None => {
                if dev_mode {
                    2
                } else {
                    100
                }
            }
        };

        let mut client_builder = GithubWorkflowClientBuilder::new();
        if dev_mode {
            client_builder = client_builder
                .base_url("https://localhost:8443")
                .ignore_trust();
        }

        let client = match client_builder
            .token(token)
            .owner(owner)
            .repo(repo)
            .timeout(Duration::from_secs(timeout))
            .retry(retry)
            .user_agent(AppMetadata::user_agent(&self.meta))
            .workflow_id(workflow)
            .since(since.clone())
            .per_page(per_page)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                errors.push(PaginatedError {
                    error_type: ErrorType::System,
                    page: None,
                    message: e.to_string(),
                });
                output_data(
                    &self.args,
                    (&[] as &[GithubWorkflowRunResponse]).to_vec(),
                    errors,
                )?;
                std::process::exit(1);
            }
        };

        let initial_response = match client.get::<GithubWorkflowRunsResponse>().await {
            Ok(r) => r,
            Err(e) => {
                errors.push(PaginatedError {
                    error_type: ErrorType::System,
                    page: None,
                    message: e.to_string(),
                });
                output_data(
                    &self.args,
                    (&[] as &[GithubWorkflowRunResponse]).to_vec(),
                    errors,
                )?;
                std::process::exit(1);
            }
        };

        let mut all_runs = initial_response.workflow_runs;

        let total_count = initial_response.total_count;

        if total_count == 0 {
            output_data(&self.args, all_runs, errors)?;
            return Ok(());
        }

        let pagination = calculate_pagination(total_count, per_page.into(), limit);

        let (more_runs, errors) = handle_paginated_requests(
            pagination,
            concurrency,
            self.args.output,
            move |page, per_page_for_page| {
                let mut client = client.clone();
                client.page = page;
                client.per_page = per_page_for_page.min(u16::MAX as usize) as u16;
                async move { client.get::<GithubWorkflowRunsResponse>().await }
            },
        )
        .await;

        all_runs.extend(more_runs.into_iter().flat_map(|r| r.workflow_runs));

        if let Some(limit) = limit {
            all_runs.truncate(limit.into());
        }

        let filtered_runs = filter_runs(&all_runs, &self.args.contains);

        let mut sorted_runs = sort_runs(filtered_runs);

        apply_user_sort(&mut sorted_runs, sort, sort_order);

        let final_runs = match head {
            Some(n) => sorted_runs.into_iter().take(n as usize).collect(),
            None => sorted_runs,
        };

        output_data(&self.args, final_runs, errors).unwrap();
        Ok(())
    }
}
