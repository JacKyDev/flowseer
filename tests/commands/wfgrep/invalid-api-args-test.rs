use flowseer::commands::wfgrep;

use flowseer::AppMetadata;
use flowseer::commands::Command;
use flowseer::commands::wfgrep::WfGrepArgs;
use flowseer::commands::wfgrep::WfGrepCommand;

#[tokio::main]
async fn test_owner_not_exist() {

    let meta = AppMetadata {
        package_name: "Alpha",
        pub version: "Beta",
        pub bin_name: "Ceta",
    };
    let args = WfGrepArgs {
        dev_mode: true
    };
    let cmd = WfGrepCommand { args, meta };

    cmd.run().await;
}
