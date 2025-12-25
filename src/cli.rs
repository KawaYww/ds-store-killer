use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Kill .DS_Store files on macOS
#[derive(Parser)]
#[command(name = "dsk", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Shared arguments for kill/watch commands
#[derive(clap::Args, Clone)]
pub struct KillArgs {
    /// Target directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Recursive deletion
    #[arg(short, long)]
    pub recursive: bool,

    /// Exclude patterns
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Skip confirmation, delete directly
    #[arg(short, long)]
    pub yes: bool,

    /// Dry-run: scan only, don't delete
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Quiet mode: suppress file listing
    #[arg(short, long)]
    pub quiet: bool,

    /// Show execution statistics
    #[arg(long)]
    pub stats: bool,

    /// Force delete git-tracked files (default: skip them)
    #[arg(long)]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Kill .DS_Store files (default command)
    #[command(name = "kill")]
    Kill {
        #[command(flatten)]
        args: KillArgs,
    },

    /// Watch directory and auto-delete .DS_Store files
    Watch {
        /// Target directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Exclude patterns
        #[arg(short, long)]
        exclude: Vec<String>,
    },

    /// Manage launchd service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// Install launchd plist
    Install {
        #[arg(default_value = "~")]
        paths: Vec<String>,
    },
    /// Uninstall launchd plist
    Uninstall,
    /// Start service
    Start,
    /// Stop service
    Stop,
    /// Show status
    Status,
}
