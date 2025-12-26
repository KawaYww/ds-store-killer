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

    /// Force delete git-tracked .DS_Store files (default: skip them)
    #[arg(long)]
    pub force: bool,
}

/// Arguments for watch command
#[derive(clap::Args, Clone)]
pub struct WatchArgs {
    /// Target directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Shared watch options
    #[command(flatten)]
    pub options: WatchSharedArgs,
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
        #[command(flatten)]
        args: WatchArgs,
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
        /// Directories to watch
        #[arg(default_value = "~")]
        paths: Vec<String>,

        #[command(flatten)]
        watch_args: WatchSharedArgs,
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

/// Arguments shared between Watch command and Service Install
#[derive(clap::Args, Clone)]
pub struct WatchSharedArgs {
    /// Exclude patterns
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Send macOS notification on delete
    #[arg(long)]
    pub notify: bool,

    /// Force delete git-tracked .DS_Store files
    #[arg(long)]
    pub force: bool,
}
