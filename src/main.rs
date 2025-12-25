mod cli;
mod consts;
mod killer;
mod log;
mod service;
mod watcher;

use clap::Parser;
use cli::{Cli, Commands, ServiceAction};
use killer::KillOptions;
use std::{io::{self, Write}, path::Path};

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Service { action }) = cli.command {
        let result = match action {
            ServiceAction::Install { paths } => service::install(&paths),
            ServiceAction::Uninstall => service::uninstall(),
            ServiceAction::Start => service::start(),
            ServiceAction::Stop => service::stop(),
            ServiceAction::Status => service::status(),
        };
        if let Err(e) = result {
            log::error(&e);
            std::process::exit(1);
        }
        return;
    }

    let path = shellexpand::tilde(&cli.path.to_string_lossy()).to_string();
    let path = Path::new(&path);
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !path.is_dir() {
        log::error(&format!("Not a directory: {}", path.display()));
        std::process::exit(1);
    }

    if cli.serve {
        if let Err(e) = watcher::run(&[path.as_path()], cli.exclude) {
            log::error(&e);
            std::process::exit(1);
        }
        return;
    }

    // Scan for .DS_Store files
    let files = killer::scan(&path, cli.recursive, &cli.exclude);

    if files.is_empty() {
        log::info("No .DS_Store files found");
        return;
    }

    // Show what will be deleted (unless quiet)
    if !cli.quiet && !cli.yes {
        log::info(&format!("Found {} .DS_Store file(s):", files.len()));
        for f in &files {
            println!("  {}", f.display());
        }
    }

    // Dry-run mode
    if cli.dry_run {
        log::info(&format!("Dry-run: {} file(s) would be deleted", files.len()));
        return;
    }

    // Confirm unless -y flag
    if !cli.yes && !confirm("Delete these files?") {
        log::info("Cancelled");
        return;
    }

    // Kill
    let opts = KillOptions {
        dry_run: false,
        quiet: cli.quiet,
    };
    let result = killer::kill(&files, &opts);

    // Output result
    if result.deleted > 0 {
        log::ok(&format!("Deleted {} .DS_Store file(s)", result.deleted));
    }

    // Stats
    if cli.stats {
        println!("  Time: {:?}", result.duration);
    }
}

/// Prompt user for confirmation
fn confirm(msg: &str) -> bool {
    print!("{} [y/N] ", msg);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
