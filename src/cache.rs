//! Cache mechanism for scan results
//!
//! Cache strategy for absolute correctness:
//! 1. Store found files with their paths + directory mtime
//! 2. On load: verify directory hasn't been modified (mtime check)
//! 3. Also verify each cached file still exists
//! 4. Auto-invalidate if directory changed (no need for --refresh)

use std::{
    env,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const CACHE_SUBDIR: &str = "dsk-cache";
const CACHE_TTL_SECS: u64 = 3600; // 1 hour (mtime check provides freshness)

/// Get cache directory path
fn cache_dir() -> PathBuf {
    env::temp_dir().join(CACHE_SUBDIR)
}

/// Generate cache key from path and recursive flag
fn cache_key(dir: &Path, recursive: bool) -> String {
    let path_hash = dir.to_string_lossy().replace('/', "_");
    format!("{}_r{}", path_hash, recursive as u8)
}

/// Get cache file path
fn cache_path(dir: &Path, recursive: bool) -> PathBuf {
    cache_dir().join(cache_key(dir, recursive))
}

/// Current unix timestamp
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// Get directory's modification time as unix timestamp
fn dir_mtime(dir: &Path) -> Option<u64> {
    fs::metadata(dir)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// Save scan results to cache
pub fn save(dir: &Path, recursive: bool, files: &[PathBuf]) {
    if fs::create_dir_all(cache_dir()).is_err() {
        return;
    }

    let path = cache_path(dir, recursive);
    let Ok(mut file) = fs::File::create(&path) else { return };

    let mtime = dir_mtime(dir).unwrap_or(0);

    // Format: line 1 = timestamp, line 2 = dir_mtime, rest = file paths
    let _ = writeln!(file, "{}", now_secs());
    let _ = writeln!(file, "{}", mtime);
    for f in files {
        let _ = writeln!(file, "{}", f.display());
    }
}

/// Load and verify cached results
/// Returns None if:
/// - Cache is missing or corrupted
/// - Cache TTL expired
/// - Directory mtime changed (new files may exist)
/// - Recursive mode (can't reliably detect subdirectory changes)
/// Returns verified files only (files that still exist)
pub fn load_verified(dir: &Path, recursive: bool) -> Option<Vec<PathBuf>> {
    // IMPORTANT: Don't cache recursive scans - subdirectory changes are undetectable
    // Only cache single-directory (non-recursive) scans where mtime check is reliable
    if recursive {
        return None;
    }

    let path = cache_path(dir, recursive);
    let file = fs::File::open(&path).ok()?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Parse timestamp
    let timestamp: u64 = lines.next()?.ok()?.parse().ok()?;

    // Parse stored mtime
    let stored_mtime: u64 = lines.next()?.ok()?.parse().ok()?;

    // Check TTL
    if now_secs() - timestamp > CACHE_TTL_SECS {
        let _ = fs::remove_file(&path);
        return None;
    }

    // Check if directory was modified (new files may have been created)
    let current_mtime = dir_mtime(dir).unwrap_or(0);
    if current_mtime != stored_mtime {
        let _ = fs::remove_file(&path);
        return None; // Directory changed, need fresh scan
    }

    // Load file paths
    let files: Vec<PathBuf> = lines
        .filter_map(|l| l.ok())
        .map(PathBuf::from)
        .collect();

    if files.is_empty() {
        return Some(vec![]);
    }

    // Verify each file still exists
    let verified: Vec<PathBuf> = files
        .into_iter()
        .filter(|f| f.exists())
        .collect();

    Some(verified)
}

/// Invalidate cache for a directory
pub fn invalidate(dir: &Path, recursive: bool) {
    let path = cache_path(dir, recursive);
    let _ = fs::remove_file(path);
}
