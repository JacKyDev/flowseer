use async_trait::async_trait;
use clap::ValueEnum;
use serde::Deserialize;
use serde::Serialize;
use tabled::Tabled;

#[async_trait]
pub trait Command {
    async fn run(&self) -> anyhow::Result<()>;
}

#[derive(Serialize, Debug)]
pub enum CLIStatus {
    SUCCESS,
    FAILED,
    PARTIAL,
}

#[derive(Serialize, Debug)]
pub struct CLIOutputData<'a, T, M, E> {
    pub status: CLIStatus,
    pub entries: &'a [T],
    pub errors: &'a [E],
    pub total: &'a usize,
    pub meta: M,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorkflowMetaData {
    pub owner: String,
    pub repo: String,
    pub workflow: String,
    pub contains: Vec<String>,
    pub token: String,
    pub output: OutputFormat,
}

#[derive(Tabled)]
pub struct WorkflowTableRowData {
    #[tabled(rename = "User")]
    pub user: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Conclusion")]
    pub conclusion: String,
    #[tabled(rename = "Duration")]
    pub duration: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Id")]
    pub id: u64,
    #[tabled(rename = "Trigger")]
    pub trigger: String,
    #[tabled(rename = "Start Date")]
    pub start_date: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Json,
}
