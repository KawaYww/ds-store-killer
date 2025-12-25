//! Colored logging utilities

use colored::Colorize;

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
pub fn kill(path: &std::path::Path) {
    println!("{} {}", "[kill]".red(), path.display());
}

#[inline]
pub fn dry(path: &std::path::Path) {
    println!("{} {}", "[dry]".magenta(), path.display());
}

#[inline]
pub fn watch(msg: &str) {
    println!("{} {}", "[watch]".cyan(), msg);
}
