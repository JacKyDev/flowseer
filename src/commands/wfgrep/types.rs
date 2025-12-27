use crate::AppMetadata;
use crate::commands::OutputFormat;
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

    pub output: OutputFormat,
    pub contains: Vec<String>,

    pub concurrency: u8,
    pub timeout: u64,
    pub retry: bool,
    pub dev_mode: bool,
}
