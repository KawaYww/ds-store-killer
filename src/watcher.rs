//! File system watcher for daemon mode

use crate::{consts::TARGET_FILE, log};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind, event::CreateKind};
use std::{fs, path::Path, sync::mpsc::channel};

/// Check if path matches any exclude pattern
#[inline]
fn is_excluded(path: &Path, excludes: &[String]) -> bool {
    let s = path.to_string_lossy();
    excludes.iter().any(|p| s.contains(p))
}

/// Watch directories and auto-delete target files
pub fn run(paths: &[&Path], excludes: Vec<String>) -> Result<(), String> {
    for p in paths {
        if !p.is_dir() {
            return Err(format!("Not a directory: {}", p.display()));
        }
    }

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| e.to_string())?;

    for p in paths {
        watcher.watch(p, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
    }

    log::watch("Watching for .DS_Store files...");
    for p in paths {
        println!("  {}", p.display());
    }
    println!("Press Ctrl+C to stop.");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if !matches!(event.kind, EventKind::Create(CreateKind::File | CreateKind::Any)) {
                    continue;
                }
                for path in event.paths {
                    if path.file_name().map_or(false, |n| n == TARGET_FILE)
                        && !is_excluded(&path, &excludes)
                    {
                        log::kill(&path);
                        if let Err(e) = fs::remove_file(&path) {
                            log::warn(&format!("Failed to remove: {}", e));
                        }
                    }
                }
            }
            Ok(Err(e)) => log::error(&e.to_string()),
            Err(_) => break,
        }
    }
    Ok(())
}
