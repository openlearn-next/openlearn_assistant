use crate::service;
use crate::settings;
use std::fs;
use std::process::Command;
#[cfg(feature = "offline")]
use std::path::PathBuf;

/// Detect the best available package manager: pnpm handles `workspace:*`
/// protocol that openlearn-next dependencies may contain; npm as fallback.
fn pkg_manager() -> &'static str {
    if Command::new("pnpm").arg("--version").output().is_ok_and(|o| o.status.success()) {
        return "pnpm";
    }
    "npm"
}

fn npm_user_prefix() -> String {
    let home = settings::home_dir();
    home.join(".local").to_string_lossy().into_owned()
}

/// Build a Command pre-configured for the detected package manager.
fn pm_cmd(subcommand: &str) -> Command {
    let pm = pkg_manager();
    let mut cmd = Command::new(pm);
    if pm == "npm" {
        cmd.arg("--prefix").arg(npm_user_prefix());
    }
    cmd.arg(subcommand);
    cmd
}

/// Path of the global bin directory.
pub fn npm_global_bin() -> Option<String> {
    let out = pm_cmd("bin").arg("-g").output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Currently installed `openlearn-next` version, if any.
pub fn installed_version() -> Option<String> {
    let out = pm_cmd("ls").arg("-g").arg("openlearn-next").arg("--depth=0").output().ok()?;
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
        run_pm_install(&tmp.display().to_string())
    }
    #[cfg(not(feature = "offline"))]
    {
        run_pm_install("openlearn-next")
    }
}

pub fn uninstall_pkg(keep_data: bool) -> Result<(), String> {
    run_pm_global("remove", &["openlearn-next"])?;
    if !keep_data {
        let dir = settings::home_dir().join("openlearn-next");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(settings::log_path());
    }
    Ok(())
}

pub fn upgrade_pkg() -> Result<(), String> {
    let _ = service::stop_service();
    run_pm_global("update", &["openlearn-next"])?;
    service::start_service()?;
    Ok(())
}

fn run_pm_install(pkg: &str) -> Result<(), String> {
    let pm = pkg_manager();
    let mut cmd = pm_cmd(if pm == "pnpm" { "add" } else { "install" });
    cmd.arg("-g").arg(pkg);
    let status = cmd.status().map_err(|e| format!("安装失败: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("安装退出码: {}", status.code().unwrap_or(-1)))
    }
}

fn run_pm_global(subcommand: &str, args: &[&str]) -> Result<(), String> {
    let mut cmd = pm_cmd(subcommand);
    cmd.arg("-g").args(args);
    let status = cmd.status().map_err(|e| format!("{subcommand} 失败: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{subcommand} 退出码: {}", status.code().unwrap_or(-1)))
    }
}

#[cfg(feature = "offline")]
fn write_embedded_openlearn() -> Result<PathBuf, String> {
    let bytes = include_bytes!("../resources/openlearn-next.tgz");
    let tmp = env::temp_dir().join("openlearn-next-offline.tgz");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    Ok(tmp)
}
