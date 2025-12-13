use crate::AppMetadata;
use crate::commands::wfgrep::{ErrorType, PaginatedError, WfGrepArgs};
use crate::commands::{
    CLIOutputData, CLIStatus, Command, OutputFormat, WorkflowMetaData, WorkflowTableRowData,
};
use crate::github::{
    GithubWorkflowClientBuilder, GithubWorkflowRunResponse, GithubWorkflowRunsResponse,
};
use crate::util::{
    convert_struct_to_table, convert_struct_to_table_with_keys, format_duration_from_str,
    mask_token, print_json, print_lines, print_table, wrap_at_spaces,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::Duration;

#[cfg(not(debug_assertions))]
use colored::*;

pub struct WfGrepCommand {
    pub args: WfGrepArgs,
    pub meta: AppMetadata,
}

impl From<&GithubWorkflowRunResponse> for WorkflowTableRowData {
    fn from(run: &GithubWorkflowRunResponse) -> Self {
        WorkflowTableRowData {
            user: run
                .actor
                .as_ref()
                .map_or("(no user)", |a| &a.login)
                .to_string(),
            status: run.status.as_deref().unwrap_or("(no status)").to_string(),
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
            trigger: run.event.as_deref().unwrap_or("unknown").to_string(),
            start_date: run
                .run_started_at
                .as_deref()
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

impl From<&WfGrepArgs> for WorkflowMetaData {
    fn from(args: &WfGrepArgs) -> Self {
        WorkflowMetaData {
            contains: args.contains.clone(),
            output: args.output,
            owner: args.owner.clone(),
            repo: args.repo.clone(),
            workflow: args.workflow.clone(),
            token: mask_token(&args.token, Some("ghp_")),
        }
    }
}

#[async_trait]
impl Command for WfGrepCommand {
    async fn run(&self) -> Result<()> {
        let WfGrepArgs {
            ref token,
            ref owner,
            ref repo,
            timeout,
            retry,
            ref workflow,
            ref concurrency,
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

        let per_page = if dev_mode {
            2
        } else {
            Some(100).map_or(100, |v| std::cmp::min(v, 100))
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

        let total_pages = total_count.div_ceil(per_page.into());

        let (more_runs, errors) =
            handle_paginated_requests(total_pages, concurrency, self.args.output, move |page| {
                let mut client = client.clone();
                client.page = page;
                async move { client.get::<GithubWorkflowRunsResponse>().await }
            })
            .await;

        all_runs.extend(more_runs.into_iter().flat_map(|r| r.workflow_runs));

        let filtered_runs = filter_runs(&all_runs, &self.args.contains);

        let final_runs = sort_runs(filtered_runs);

        output_data(&self.args, final_runs, errors).unwrap();
        Ok(())
    }
}

pub async fn handle_paginated_requests<T, E, F, Fut>(
    total_pages: usize,
    concurrency: &u8,
    format: OutputFormat,
    request_fn: F,
) -> (Vec<T>, Vec<PaginatedError>)
where
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
    F: Fn(usize) -> Fut + Send + Sync + 'static + Clone,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
{
    let mut results = Vec::new();
    let mut errors = Vec::new();

    let show_progress = matches!(format, OutputFormat::Table);
    let pb = if show_progress {
        let pb = ProgressBar::new((total_pages - 1) as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Seiten",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let semaphore = Arc::new(Semaphore::new(*concurrency as usize));
    let mut tasks = FuturesUnordered::new();

    for page in 2..=total_pages {
        let permit = semaphore.clone();
        let request_fn = request_fn.clone();
        let pb = pb.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            let res = request_fn(page).await;
            if let Some(pb) = pb {
                pb.inc(1);
            }
            (page, res)
        }));
    }

    while let Some(joined) = tasks.next().await {
        match joined {
            Ok((_page, Ok(result))) => results.push(result),
            Ok((page, Err(e))) => {
                errors.push(PaginatedError {
                    error_type: ErrorType::Process,
                    page: Some(page),
                    message: e.to_string(),
                });
            }
            Err(e) => {
                errors.push(PaginatedError {
                    error_type: ErrorType::System,
                    page: None,
                    message: format!("Join-Fehler: {}", e),
                });
            }
        }
    }

    if let Some(pb) = &pb {
        pb.finish_with_message("Alle Seiten geladen.");
    }

    (results, errors)
}

fn output_data(
    args: &WfGrepArgs,
    runs: Vec<GithubWorkflowRunResponse>,
    mut errors: Vec<PaginatedError>,
) -> Result<()> {
    match args.output {
        OutputFormat::Table => {
            if runs.is_empty() {
                print_lines("No runs found matching filters.", "Results");
            } else {
                let rows: Vec<WorkflowTableRowData> = runs.iter().map(|run| run.into()).collect();
                print_table(&rows, "Results");
            }

            let meta_data: WorkflowMetaData = args.into();
            let meta_raw = convert_struct_to_table(&meta_data);
            print_table(&meta_raw, "Metadata");

            if !errors.is_empty() {
                errors.sort_by_key(|e| e.page.unwrap_or(usize::MAX));

                let error_raw = convert_struct_to_table_with_keys(&errors, "page", "message");
                print_table(&error_raw, "Errors");
            }
            Ok(())
        }
        OutputFormat::Json => {
            let meta_data: WorkflowMetaData = args.into();
            let mut status = CLIStatus::SUCCESS;
            if !errors.is_empty() {
                if runs.is_empty() {
                    errors.sort_by_key(|e| e.page.unwrap_or(usize::MAX));
                    status = CLIStatus::FAILED;
                } else {
                    errors.sort_by_key(|e| e.page.unwrap_or(usize::MAX));
                    status = CLIStatus::PARTIAL;
                }
            }

            print_json(
                CLIOutputData {
                    status,
                    entries: &runs,
                    total: &runs.len(),
                    meta: meta_data,
                    errors: &errors,
                },
                None,
            );
            Ok(())
        }
    }
}

fn filter_runs(
    runs: &[GithubWorkflowRunResponse],
    filters: &[String],
) -> Vec<GithubWorkflowRunResponse> {
    if filters.is_empty() {
        return runs.to_vec();
    }

    runs.iter()
        .filter(|run| {
            run.name
                .as_ref()
                .is_some_and(|name| filters.iter().all(|f| name.contains(f)))
        })
        .cloned()
        .collect()
}

fn sort_runs(mut runs: Vec<GithubWorkflowRunResponse>) -> Vec<GithubWorkflowRunResponse> {
    runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    runs
}

#[cfg(not(debug_assertions))]
fn print_dev_mode_warning() {
    eprintln!("{}", "WARNING: Development mode is ENABLED!".bold().red());
    eprintln!(
        "{}",
        "This mode is intended for development ONLY and MUST NOT be used in any production environment."
            .yellow()
    );
    eprintln!(
        "{}",
        "Security mechanisms are disabled, and certain validations may be bypassed.".yellow()
    );
    eprintln!();
}

#[cfg(not(debug_assertions))]
fn print_dev_mode_impacts() {
    println!(
        "{}",
        "Development mode is active – the following changes apply:"
            .bold()
            .green()
    );
    println!(" - TLS certificate verification is disabled");
    println!(" - Some security features are turned off");
    println!(" - https://localhost:8443 used as API Request Url");
    println!(" - Default per-page elements: 2");
    println!();
}
