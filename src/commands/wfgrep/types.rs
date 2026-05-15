use crate::AppMetadata;
use crate::commands::OutputFormat;
use crate::commands::SortOrder;
use crate::commands::types::Sort;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum ErrorType {
    System,
    Process,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedError {
    pub error_type: ErrorType,
    pub page: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub total_pages: usize,
    pub effective_limit: usize,
    pub last_page_size: usize,
    pub per_page: usize,
}

pub struct WfGrepCommand {
    pub args: EffectiveConfig,
    pub meta: AppMetadata,
}

#[derive(Debug)]
pub struct EffectiveConfig {
    pub repo: String,
    pub owner: String,
    pub workflow: String,
    pub token: String,

    pub limit: Option<u16>,
    pub head: Option<u16>,

    pub output: OutputFormat,
    pub contains: Vec<String>,

    pub concurrency: u8,
    pub timeout: u64,
    pub retry: bool,
    pub dev_mode: bool,

    pub since: Option<String>,

    pub sort_order: SortOrder,
    pub sort: Sort,
}
