//! Git safety checks for .DS_Store files
//!
//! Detects if a file is tracked by git to prevent breaking commit history.

use std::path::Path;
use std::process::Command;

/// Check if git command is available on the system
pub fn is_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a file is tracked by git (would affect commit history if deleted)
pub fn is_git_tracked(path: &Path) -> bool {
    // First check if we're in a git repo
    let Some(parent) = path.parent() else { return false };

    // Use git ls-files to check if file is tracked
    Command::new("git")
        .args(["ls-files", "--error-unmatch"])
        .arg(path)
        .current_dir(parent)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Result of git safety check
#[derive(Debug)]
pub struct SafetyResult {
    pub safe: Vec<std::path::PathBuf>,      // Files safe to delete
    pub tracked: Vec<std::path::PathBuf>,   // Files tracked by git
}

/// Check multiple files for git tracking
pub fn check_files(files: &[std::path::PathBuf]) -> SafetyResult {
    let mut safe = Vec::new();
    let mut tracked = Vec::new();

    for file in files {
        if is_git_tracked(file) {
            tracked.push(file.clone());
        } else {
            safe.push(file.clone());
        }
    }

    SafetyResult { safe, tracked }
}
