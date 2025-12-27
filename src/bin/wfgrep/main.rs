use clap::Parser;
use flowseer::AppMetadata;
use flowseer::commands::Command;
use flowseer::commands::wfgrep::WfGrepArgs;
use flowseer::commands::wfgrep::WfGrepCommand;
use flowseer::commands::wfgrep::types::EffectiveConfig;
use flowseer::profile::load_profile;
use flowseer::util::resolve_home_dir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let meta = AppMetadata::from_bin(env!("CARGO_BIN_NAME"));
    let args = WfGrepArgs::parse();

    let home = resolve_home_dir()?;
    let profile = match &args.profile {
        Some(name) => Some(load_profile(name, &home)?),
        None => None,
    };

    let config = EffectiveConfig::from_args_and_profile(args, profile)?;

    let cmd = WfGrepCommand { args: config, meta };

    cmd.run().await
}
