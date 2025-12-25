//! launchd service management

use crate::{consts::*, log};
use std::{env, fs, io::Write, path::PathBuf, process::Command};

fn home_dir() -> PathBuf {
    shellexpand::tilde("~").to_string().into()
}

fn plist_path() -> PathBuf {
    home_dir().join("Library/LaunchAgents").join(PLIST_FILENAME)
}

fn expand(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

fn generate_plist(exe: &str, paths: &[String]) -> String {
    let args: String = paths
        .iter()
        .map(|p| format!("        <string>{}</string>", expand(p)))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{SERVICE_ID}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>--serve</string>
{args}
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{LOG_STDOUT}</string>
    <key>StandardErrorPath</key>
    <string>{LOG_STDERR}</string>
</dict>
</plist>
"#)
}

pub fn install(paths: &[String]) -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let plist = plist_path();

    if let Some(parent) = plist.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let watch: Vec<String> = if paths.is_empty() {
        vec!["~".into()]
    } else {
        paths.to_vec()
    };

    let content = generate_plist(&exe.to_string_lossy(), &watch);
    fs::File::create(&plist)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| e.to_string())?;

    log::ok(&format!("Installed plist to: {}", plist.display()));
    for p in &watch {
        println!("  - {}", expand(p));
    }
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let _ = stop();
    let plist = plist_path();

    if plist.exists() {
        fs::remove_file(&plist).map_err(|e| e.to_string())?;
        log::ok(&format!("Uninstalled: {}", plist.display()));
    } else {
        log::info(&format!("Not installed: {}", plist.display()));
    }
    Ok(())
}

pub fn start() -> Result<(), String> {
    let plist = plist_path();
    if !plist.exists() {
        return Err(format!("Not installed. Run '{} service install' first.", APP_NAME));
    }

    let out = Command::new("launchctl")
        .args(["load", "-w", &plist.to_string_lossy()])
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        log::ok("Service started");
        println!("  Logs: {}, {}", LOG_STDOUT, LOG_STDERR);
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("already loaded") {
            log::info("Service already running");
        } else {
            return Err(err.to_string());
        }
    }
    Ok(())
}

pub fn stop() -> Result<(), String> {
    let plist = plist_path();
    if !plist.exists() {
        log::info("Service not installed");
        return Ok(());
    }

    let out = Command::new("launchctl")
        .args(["unload", &plist.to_string_lossy()])
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        log::ok("Service stopped");
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("Could not find") || err.contains("not loaded") {
            log::info("Service not running");
        } else {
            return Err(err.to_string());
        }
    }
    Ok(())
}

pub fn status() -> Result<(), String> {
    let plist = plist_path();

    println!("Service:   {}", SERVICE_ID);
    println!("Plist:     {}", plist.display());
    println!("Installed: {}", if plist.exists() { "Yes" } else { "No" });

    let out = Command::new("launchctl")
        .args(["list", SERVICE_ID])
        .output()
        .map_err(|e| e.to_string())?;

    println!("Running:   {}", if out.status.success() { "Yes" } else { "No" });
    println!("\nLogs:");
    println!("  stdout: {}", LOG_STDOUT);
    println!("  stderr: {}", LOG_STDERR);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand() {
        assert!(expand("~/test").contains("/test"));
        assert_eq!(expand("/abs/path"), "/abs/path");
    }

    #[test]
    fn test_plist_content() {
        let plist = generate_plist("/bin/dsk", &["~".into()]);
        assert!(plist.contains(SERVICE_ID));
        assert!(plist.contains("--serve"));
    }
}
