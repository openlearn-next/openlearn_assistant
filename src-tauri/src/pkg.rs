use crate::elevate;
use crate::service;
use crate::settings;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Path of the global `npm bin` directory (where the `openlearn-next` shim lives).
pub fn npm_global_bin() -> Option<String> {
    let out = Command::new("npm").args(["bin", "-g"]).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Currently installed `openlearn-next` version, if any.
pub fn installed_version() -> Option<String> {
    let out = Command::new("npm")
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
        elevate::run_elevated(&format!("npm install -g '{}'", tmp.display()))
    }
    #[cfg(not(feature = "offline"))]
    {
        elevate::run_elevated("npm install -g openlearn-next")
    }
}

pub fn uninstall_pkg(keep_data: bool) -> Result<(), String> {
    elevate::run_elevated("npm uninstall -g openlearn-next")?;
    if !keep_data {
        let dir = settings::home_dir().join("openlearn-next");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(settings::log_path());
    }
    Ok(())
}

pub fn upgrade_pkg() -> Result<(), String> {
    // stop if running, update, then start again
    let _ = service::stop_service();
    elevate::run_elevated("npm update -g openlearn-next")?;
    service::start_service()?;
    Ok(())
}

#[cfg(feature = "offline")]
fn write_embedded_openlearn() -> Result<PathBuf, String> {
    let bytes = include_bytes!("../resources/openlearn-next.tgz");
    let tmp = env::temp_dir().join("openlearn-next-offline.tgz");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    Ok(tmp)
}
