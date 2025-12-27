use crate::commands::CLIOutputData;
use crate::commands::CLIStatus;
use crate::commands::OutputFormat;
use crate::commands::WorkflowMetaData;
use crate::commands::WorkflowTableRowData;
use crate::commands::wfgrep::EffectiveConfig;
use crate::commands::wfgrep::ErrorType;
use crate::commands::wfgrep::PaginatedError;
use crate::github::GithubWorkflowRunResponse;
use crate::util::convert_struct_to_table;
use crate::util::convert_struct_to_table_with_keys;
use crate::util::print_json;
use crate::util::print_lines;
use crate::util::print_table;
use anyhow::Result;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[cfg(not(debug_assertions))]
use colored::*;

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

pub fn output_data(
    args: &EffectiveConfig,
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

pub fn filter_runs(
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

pub fn sort_runs(mut runs: Vec<GithubWorkflowRunResponse>) -> Vec<GithubWorkflowRunResponse> {
    runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    runs
}

#[cfg(not(debug_assertions))]
pub fn print_dev_mode_warning() {
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
pub fn print_dev_mode_impacts() {
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
