//! Application-wide constants

/// Application name
pub const APP_NAME: &str = "dsk";

/// Target filename to kill
pub const TARGET_FILE: &str = ".DS_Store";

/// launchd service identifier
pub const SERVICE_ID: &str = "com.dsk.guard";

/// Log output paths
pub const LOG_STDOUT: &str = "/tmp/dsk.out.log";
pub const LOG_STDERR: &str = "/tmp/dsk.err.log";

/// launchd plist filename
pub const PLIST_FILENAME: &str = "com.dsk.guard.plist";
