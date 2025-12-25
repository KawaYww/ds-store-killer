use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dsk", version, about = "Kill .DS_Store files on macOS")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Target directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Recursive deletion
    #[arg(short, long)]
    pub recursive: bool,

    /// Daemon mode: watch and auto-delete
    #[arg(long)]
    pub serve: bool,

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
}

#[derive(Subcommand)]
pub enum Commands {
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
