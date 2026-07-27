use crate::service;
use crate::settings;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// User-local npm prefix so install/uninstall/upgrade never need root.
fn npm_user_prefix() -> String {
    let home = settings::home_dir();
    home.join(".local").to_string_lossy().into_owned()
}

/// Path of the user-local npm `bin` directory.
pub fn npm_global_bin() -> Option<String> {
    let prefix = npm_user_prefix();
    let out = Command::new("npm")
        .arg("--prefix")
        .arg(&prefix)
        .args(["bin", "-g"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Currently installed `openlearn-next` version, if any.
pub fn installed_version() -> Option<String> {
    let prefix = npm_user_prefix();
    let out = Command::new("npm")
        .arg("--prefix")
        .arg(&prefix)
        .args(["ls", "-g", "openlearn-next", "--depth=0"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if let Some(idx) = line.find("openlearn-next@") {
            let rest = &line[idx + "openlearn-next@".len()..];
            let ver = rest.trim().split_whitespace().next().unwrap_or("");
            if !ver.is_empty() {
                return Some(ver.to_string());
            }
        }
    }
    None
}

pub fn install_pkg() -> Result<(), String> {
    #[cfg(feature = "offline")]
    {
        let tmp = write_embedded_openlearn()?;
        run_npm(&["install", "-g", &tmp.display().to_string()])
    }
    #[cfg(not(feature = "offline"))]
    {
        run_npm(&["install", "-g", "openlearn-next"])
    }
}

pub fn uninstall_pkg(keep_data: bool) -> Result<(), String> {
    run_npm(&["uninstall", "-g", "openlearn-next"])?;
    if !keep_data {
        let dir = settings::home_dir().join("openlearn-next");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(settings::log_path());
    }
    Ok(())
}

pub fn upgrade_pkg() -> Result<(), String> {
    let _ = service::stop_service();
    run_npm(&["update", "-g", "openlearn-next"])?;
    service::start_service()?;
    Ok(())
}

fn run_npm(args: &[&str]) -> Result<(), String> {
    let prefix = npm_user_prefix();
    let status = Command::new("npm")
        .arg("--prefix")
        .arg(&prefix)
        .args(args)
        .status()
        .map_err(|e| format!("npm 命令失败: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("npm 命令退出码: {}", status.code().unwrap_or(-1)))
    }
}

#[cfg(feature = "offline")]
fn write_embedded_openlearn() -> Result<PathBuf, String> {
    let bytes = include_bytes!("../resources/openlearn-next.tgz");
    let tmp = env::temp_dir().join("openlearn-next-offline.tgz");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    Ok(tmp)
}
