mod cache;
mod cli;
mod consts;
mod git;
mod killer;
mod log;
mod service;
mod watcher;

use clap::Parser;
use cli::{Cli, Commands, KillArgs, ServiceAction};
use killer::KillOptions;
use std::{io::{self, Write}, path::Path};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Kill { args } => run_kill(args),
        Commands::Watch { path, exclude } => run_watch(&path, exclude),
        Commands::Service { action } => run_service(action),
    }
}

fn run_kill(args: KillArgs) {
    let path = shellexpand::tilde(&args.path.to_string_lossy()).to_string();
    let path = Path::new(&path);
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !path.is_dir() {
        log::error(&format!("Not a directory: {}", path.display()));
        std::process::exit(1);
    }

    let opts = KillOptions {
        dry_run: args.dry_run,
        quiet: args.quiet,
    };

    // Fast path: -y flag means streaming mode (no confirmation needed)
    if args.yes {
        let result = killer::kill_streaming(&path, args.recursive, &args.exclude, &opts);

        if !args.dry_run && result.deleted > 0 {
            cache::invalidate(&path, args.recursive);
        }

        if result.found == 0 {
            log::info("No .DS_Store files found");
        } else if args.dry_run {
            log::info(&format!("Dry-run: {} file(s) would be deleted", result.found));
        } else {
            log::ok(&format!("Deleted {} .DS_Store file(s)", result.deleted));
        }

        if args.stats {
            println!("  Time: {:?}", result.duration);
        }
        return;
    }

    // Interactive path: try cache first
    let files = if let Some(cached) = cache::load_verified(&path, args.recursive) {
        if cached.is_empty() {
            log::info("No .DS_Store files found (cached)");
            return;
        }
        log::info(&format!("Found {} file(s) (cached)", cached.len()));
        for f in &cached {
            log::found(f);
        }
        cached
    } else {
        log::info("Scanning for .DS_Store files...");
        scan_and_cache(&path, args.recursive, &args.exclude)
    };

    if files.is_empty() {
        log::info("No .DS_Store files found");
        return;
    }

    log::info(&format!("Found {} file(s)", files.len()));

    if args.dry_run {
        log::info(&format!("Dry-run: {} file(s) would be deleted", files.len()));
        return;
    }

    // Git safety check (only if git is available)
    let files_to_delete = if git::is_available() {
        let safety = git::check_files(&files);

        if !safety.tracked.is_empty() {
            if args.force {
                log::warn(&format!(
                    "{} file(s) are git-tracked (--force: will delete anyway)",
                    safety.tracked.len()
                ));
                for f in &safety.tracked {
                    log::found(f);
                }
            } else {
                log::warn(&format!(
                    "Skipping {} git-tracked file(s) (use --force to delete)",
                    safety.tracked.len()
                ));
            }
        }

        if args.force { files.clone() } else { safety.safe }
    } else {
        // Git not available - warn once and proceed without safety check
        log::warn("git not found - cannot check for tracked files");
        files.clone()
    };

    if files_to_delete.is_empty() {
        log::info("No files to delete (all are git-tracked, use --force)");
        return;
    }

    let msg = format!("Delete {} file(s)?", files_to_delete.len());
    if !confirm(&msg) {
        log::info("Cancelled");
        return;
    }

    let result = killer::kill_files(&files_to_delete, &KillOptions {
        dry_run: false,
        quiet: true,
    });

    cache::invalidate(&path, args.recursive);
    log::ok(&format!("Deleted {} .DS_Store file(s)", result.deleted));

    if args.stats {
        println!("  Time: {:?}", result.duration);
    }
}

fn run_watch(path: &std::path::Path, exclude: Vec<String>) {
    let path = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let path = Path::new(&path);
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !path.is_dir() {
        log::error(&format!("Not a directory: {}", path.display()));
        std::process::exit(1);
    }

    if let Err(e) = watcher::run(&[path.as_path()], exclude) {
        log::error(&e);
        std::process::exit(1);
    }
}

fn run_service(action: ServiceAction) {
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
}

fn scan_and_cache(dir: &Path, recursive: bool, excludes: &[String]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    killer::scan_streaming(dir, recursive, excludes, |p| {
        log::found(p);
        files.push(p.to_path_buf());
    });
    cache::save(dir, recursive, &files);
    files
}

fn confirm(msg: &str) -> bool {
    print!("{} [y/N] ", msg);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
