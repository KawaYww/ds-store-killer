//! Colored logging utilities

use colored::Colorize;
use std::path::Path;
use shellexpand;

/// Shorten path by replacing home directory with ~
pub fn shorten_path(path: &Path) -> String {
    let home = shellexpand::tilde("~").to_string();
    let path_str = path.to_string_lossy();

    if path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str.to_string()
    }
}

/// Format path with filename highlighted in light red
#[inline]
fn format_path(path: &Path) -> String {
    let short = shorten_path(path);
    // Find last / to highlight filename
    if let Some(pos) = short.rfind('/') {
        let (parent, name) = short.split_at(pos + 1);
        format!("{}{}", parent, name.truecolor(255, 100, 100))
    } else {
        short.truecolor(255, 100, 100).to_string()
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
