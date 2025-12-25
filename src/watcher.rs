//! File system watcher for daemon mode

use crate::{consts::TARGET_FILE, log};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::{fs, path::Path, process::Command, sync::mpsc::channel};

/// Check if path matches any exclude pattern
#[inline]
fn is_excluded(path: &Path, excludes: &[String]) -> bool {
    let s = path.to_string_lossy();
    excludes.iter().any(|p| s.contains(p))
}

/// Send macOS native notification using osascript
fn send_notification(title: &str, message: &str) {
    let script = format!(
        r#"display notification "{}" with title "{}""#,
        message.replace('"', r#"\""#),
        title.replace('"', r#"\""#)
    );
    let _ = Command::new("osascript")
        .args(["-e", &script])
        .output();
}

/// Watch directories and auto-delete target files
pub fn run(paths: &[&Path], mut excludes: Vec<String>, notify: bool, force: bool) -> Result<(), String> {
    // Add default excludes if not present
    let defaults = ["node_modules", ".git", "target"];
    for d in defaults {
        if !excludes.iter().any(|e| e == d) {
            excludes.push(d.to_string());
        }
    }

    for p in paths {
        if !p.is_dir() {
            return Err(format!("Not a directory: {}", p.display()));
        }
    }

    // Initial scan and cleanup
    log::watch("Performing initial cleanup...");

    for p in paths {
        crate::killer::scan_streaming(p, true, &excludes, |path| {
            // Check git safety mechanism
            if !force && crate::git::is_available() && crate::git::is_git_tracked(path) {
                log::warn(&format!("Skipping git-tracked file: {}", path.display()));
                return;
            }

            log::kill(path);
            if let Err(e) = fs::remove_file(path) {
                log::warn(&format!("Failed to remove: {}", e));
            } else if notify {
                send_notification("dsk", &format!("Killed {}", path.display()));
            }
        });
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
                // FSEvents might report creation as Modify or Other, so we check broadly
                if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any) {
                    continue;
                }
                for path in event.paths {
                    if path.file_name().map_or(false, |n| n == TARGET_FILE)
                        && !is_excluded(&path, &excludes)
                    {
                        // Check if file still exists (avoid "No such file" from duplicate events)
                        if !path.exists() {
                            continue;
                        }

                        // Check git safety
                        if !force && crate::git::is_available() && crate::git::is_git_tracked(&path) {
                            // Only warn once per file roughly (logs might get spammy if OS keeps retrying)
                            // But usually OS stops writing if we don't delete it.
                            // If we don't delete it, we might get fewer events.
                            log::warn(&format!("Skipping git-tracked file: {}", path.display()));
                            continue;
                        }

                        log::kill(&path);
                        if let Err(e) = fs::remove_file(&path) {
                            // Ignore "NotFound" error in case of race condition
                            if e.kind() != std::io::ErrorKind::NotFound {
                                log::warn(&format!("Failed to remove: {}", e));
                            }
                        } else if notify {
                            send_notification("dsk", &format!("Killed {}", path.display()));
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
