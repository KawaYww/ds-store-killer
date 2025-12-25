//! Core deletion logic

use crate::{consts::TARGET_FILE, log};
use jwalk::WalkDir;
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

/// Options controlling kill behavior
#[derive(Default)]
pub struct KillOptions {
    pub dry_run: bool,
    pub quiet: bool,
}

/// Result of a kill operation
pub struct KillResult {
    pub found: usize,
    pub deleted: usize,
    pub duration: Duration,
}

impl std::fmt::Display for KillResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.deleted > 0 {
            write!(f, "Deleted {} .DS_Store file(s)", self.deleted)
        } else if self.found > 0 {
            write!(f, "Found {} .DS_Store file(s)", self.found)
        } else {
            write!(f, "No .DS_Store files found")
        }
    }
}

/// Check if a path is the target file
#[inline]
pub fn is_target(path: &Path) -> bool {
    path.file_name().map_or(false, |n| n == TARGET_FILE)
}

/// Check if a path matches any exclude pattern
#[inline]
pub fn is_excluded(path: &Path, excludes: &[String]) -> bool {
    let s = path.to_string_lossy();
    excludes.iter().any(|p| s.contains(p))
}

/// Scan directory and return list of target files
pub fn scan(dir: &Path, recursive: bool, excludes: &[String]) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();

    if recursive {
        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if is_target(&path) && !is_excluded(&path, excludes) {
                found.push(path);
            }
        }
    } else {
        let target = dir.join(TARGET_FILE);
        if target.exists() && !is_excluded(&target, excludes) {
            found.push(target);
        }
    }

    found
}

/// Kill target files with given options
pub fn kill(files: &[std::path::PathBuf], opts: &KillOptions) -> KillResult {
    let start = Instant::now();
    let found = files.len();
    let deleted = AtomicUsize::new(0);

    for path in files {
        if !opts.quiet {
            if opts.dry_run {
                log::dry(path);
            } else {
                log::kill(path);
            }
        }

        if !opts.dry_run {
            if fs::remove_file(path).is_ok() {
                deleted.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    let deleted_count = if opts.dry_run { 0 } else { deleted.load(Ordering::Relaxed) };

    KillResult {
        found,
        deleted: deleted_count,
        duration: start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_target() {
        assert!(is_target(Path::new(".DS_Store")));
        assert!(is_target(Path::new("/a/b/.DS_Store")));
        assert!(!is_target(Path::new("file.txt")));
        assert!(!is_target(Path::new(".DS_Store.bak")));
    }

    #[test]
    fn test_is_excluded() {
        let ex = vec!["node_modules".into(), ".git".into()];
        assert!(is_excluded(Path::new("/a/node_modules/.DS_Store"), &ex));
        assert!(is_excluded(Path::new("/a/.git/objects"), &ex));
        assert!(!is_excluded(Path::new("/a/src/.DS_Store"), &ex));
    }

    #[test]
    fn test_kill_result_display() {
        let r = KillResult { found: 0, deleted: 0, duration: Duration::ZERO };
        assert_eq!(r.to_string(), "No .DS_Store files found");

        let r = KillResult { found: 5, deleted: 0, duration: Duration::ZERO };
        assert_eq!(r.to_string(), "Found 5 .DS_Store file(s)");

        let r = KillResult { found: 5, deleted: 5, duration: Duration::ZERO };
        assert_eq!(r.to_string(), "Deleted 5 .DS_Store file(s)");
    }
}
