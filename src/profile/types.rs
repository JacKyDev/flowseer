use crate::commands::OutputFormat;
use crate::commands::SortOrder;
use crate::commands::types::Sort;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub repo: Option<String>,
    pub owner: Option<String>,
    pub workflow: Option<String>,
    pub token: Option<String>,

    pub limit: Option<u16>,
    pub head: Option<u16>,

    pub output: Option<OutputFormat>,
    pub contains: Option<Vec<String>>,

    pub concurrency: Option<u8>,
    pub timeout: Option<u64>,
    pub retry: Option<bool>,

    pub sort: Option<Sort>,
    pub sort_order: Option<SortOrder>,
}
