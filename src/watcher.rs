//! File system watcher for daemon mode

use crate::{consts::TARGET_FILE, git, log};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{fs, path::Path, process::Command, sync::mpsc::channel};

/// Check if path matches any exclude pattern
#[inline]
fn is_excluded(path: &Path, excludes: &[String]) -> bool {
    let s = path.to_string_lossy();
    excludes.iter().any(|p| s.contains(p))
}

/// Send macOS native notification
fn send_notification(message: &str) {
    let script = format!(
        r#"display notification "{}" with title "🗑️ dsk""#,
        message.replace('"', r#"\""#)
    );
    let _ = Command::new("osascript")
        .args(["-e", &script])
        .output();
}

/// Attempt to delete a .DS_Store file with git safety check
/// Returns true if file was deleted, false if skipped
fn try_delete(path: &Path, force: bool, notify: bool) -> bool {
    // Git safety check
    if !force && git::is_available() && git::is_git_tracked(path) {
        log::warn(&format!("Skipping git-tracked: {}", path.display()));
        return false;
    }

    log::kill(path);
    match fs::remove_file(path) {
        Ok(()) => {
            if notify {
                send_notification(&format!("Killed {}", path.display()));
            }
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true, // Already gone
        Err(e) => {
            log::warn(&format!("Failed to remove: {}", e));
            false
        }
    }
}

/// Watch directories and auto-delete .DS_Store files
pub fn run(paths: &[&Path], mut excludes: Vec<String>, notify: bool, force: bool) -> Result<(), String> {
    // Add default excludes
    for d in ["node_modules", ".git", "target"] {
        if !excludes.iter().any(|e| e == d) {
            excludes.push(d.to_string());
        }
    }

    for p in paths {
        if !p.is_dir() {
            return Err(format!("Not a directory: {}", p.display()));
        }
    }

    // Initialize watcher first to capture all events (eliminates vacuum period)
    let (tx, rx) = channel();
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).map_err(|e| e.to_string())?;

    for p in paths {
        watcher
            .watch(p, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;
    }

    log::watch("Watching for .DS_Store files...");
    for p in paths {
        println!("  {}", p.display());
    }

    // Initial cleanup (events buffered in channel during scan)
    log::watch("Performing initial cleanup...");
    for p in paths {
        crate::killer::scan_streaming(p, true, &excludes, |path| {
            try_delete(path, force, notify);
        });
    }

    println!("Press Ctrl+C to stop.");

    // Event loop
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if !matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any
                ) {
                    continue;
                }

                for path in event.paths {
                    if path.file_name().map_or(false, |n| n == TARGET_FILE)
                        && !is_excluded(&path, &excludes)
                        && path.exists()
                    {
                        try_delete(&path, force, notify);
                    }
                }
            }
            Ok(Err(e)) => log::error(&e.to_string()),
            Err(_) => break,
        }
    }

    Ok(())
}

