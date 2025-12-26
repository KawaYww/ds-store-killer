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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Init git repo
        Command::new("git")
            .arg("init")
            .current_dir(&dir)
            .output()
            .expect("Failed to init git repo");

        // Config minimal user for commit
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&dir)
            .output()
            .unwrap();

        dir
    }

    #[test]
    fn test_is_git_tracked() {
        if !is_available() {
            // Skip test if git is not installed
            return;
        }

        let dir = setup_git_repo();
        let path = dir.path();

        // Create tracked file
        let tracked_file = path.join("tracked.txt");
        File::create(&tracked_file).unwrap();
        Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();

        // Create untracked file
        let untracked_file = path.join("untracked.txt");
        File::create(&untracked_file).unwrap();

        // Create ignored file
        let ignored_file = path.join("ignored.txt");
        File::create(&ignored_file).unwrap();
        let gitignore = path.join(".gitignore");
        std::fs::write(&gitignore, "ignored.txt").unwrap();

        assert!(is_git_tracked(&tracked_file), "Tracked file should be detected");
        assert!(!is_git_tracked(&untracked_file), "Untracked file should not be detected");
        assert!(!is_git_tracked(&ignored_file), "Ignored file should not be detected");

        // Test file outside repo
        let non_repo = TempDir::new().unwrap();
        let file = non_repo.path().join("file.txt");
        File::create(&file).unwrap();
        assert!(!is_git_tracked(&file), "File outside repo should not be tracked");
    }
}
