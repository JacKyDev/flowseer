use crate::commands::OutputFormat;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub repo: Option<String>,
    pub owner: Option<String>,
    pub workflow: Option<String>,
    pub token: Option<String>,

    pub output: Option<OutputFormat>,
    pub contains: Option<Vec<String>>,

    pub concurrency: Option<u8>,
    pub timeout: Option<u64>,
    pub retry: Option<bool>,
}
