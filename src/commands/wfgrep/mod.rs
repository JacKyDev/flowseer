//! Flowseer wfgrep Command
//!
//! The wfgrep definition and logical layer.

pub use args::WfGrepArgs;
pub use types::{EffectiveConfig, ErrorType, PaginatedError, WfGrepCommand};
pub use utils::{filter_runs, handle_paginated_requests, output_data, sort_runs};

#[cfg(not(debug_assertions))]
pub use utils::{print_dev_mode_impacts, print_dev_mode_warning};

pub mod args;
pub mod config;
pub mod handler;
pub mod types;
pub mod utils;
