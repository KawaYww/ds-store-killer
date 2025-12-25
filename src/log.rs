//! Colored logging utilities

use colored::Colorize;
use std::path::Path;

/// Format path with filename highlighted in light red
#[inline]
fn format_path(path: &Path) -> String {
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) if !parent.as_os_str().is_empty() => {
            format!(
                "{}/{}",
                parent.display(),
                name.to_string_lossy().truecolor(255, 100, 100) // light red
            )
        }
        _ => path.display().to_string().truecolor(255, 100, 100).to_string(),
    }
}

#[inline]
pub fn ok(msg: &str) {
    println!("{} {}", "[ok]".green(), msg);
}

#[inline]
pub fn info(msg: &str) {
    println!("{} {}", "[info]".blue(), msg);
}

#[inline]
pub fn warn(msg: &str) {
    eprintln!("{} {}", "[warn]".yellow(), msg);
}

#[inline]
pub fn error(msg: &str) {
    eprintln!("{} {}", "[error]".red(), msg);
}

#[inline]
pub fn kill(path: &Path) {
    println!("{} {}", "[kill]".red(), format_path(path));
}

#[inline]
pub fn dry(path: &Path) {
    println!("{} {}", "[dry]".magenta(), format_path(path));
}

#[inline]
pub fn watch(msg: &str) {
    println!("{} {}", "[watch]".cyan(), msg);
}

/// Display a found path with filename highlighted
#[inline]
pub fn found(path: &std::path::Path) {
    println!("  {}", format_path(path));
}
