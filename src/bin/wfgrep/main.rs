use clap::Parser;
use flowseer::AppMetadata;
use flowseer::commands::Command;
use flowseer::commands::wfgrep::WfGrepArgs;
use flowseer::commands::wfgrep::WfGrepCommand;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let meta = AppMetadata::from_bin(env!("CARGO_BIN_NAME"));
    let args = WfGrepArgs::parse();
    let cmd = WfGrepCommand { args, meta };

    cmd.run().await
}
