use crate::elevate;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct NodeInfo {
    pub installed: bool,
    pub version: String,
    pub path: String,
    /// better-sqlite3 (used by openlearn-next) requires Node >= 22.
    pub meets_requirement: bool,
}

pub fn detect_node() -> NodeInfo {
    if let Ok(out) = Command::new("node").arg("--version").output() {
        if out.status.success() {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return NodeInfo {
                installed: true,
                version: v.clone(),
                path: which_node(),
                meets_requirement: node_major(&v) >= 22,
            };
        }
    }
    NodeInfo {
        installed: false,
        version: String::new(),
        path: String::new(),
        meets_requirement: false,
    }
}

fn node_major(v: &str) -> u32 {
    v.trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

fn which_node() -> String {
    let out = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg("node")
        .output();
    out.ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Install Node 22 LTS system-wide (elevated).
pub fn provision_node() -> Result<(), String> {
    #[cfg(feature = "offline")]
    {
        provision_node_offline()
    }
    #[cfg(not(feature = "offline"))]
    {
        provision_node_online()
    }
}

#[cfg(not(feature = "offline"))]
fn provision_node_online() -> Result<(), String> {
    let ver = latest_node22()?;
    let (filename, is_zip) = node_asset_name(&ver);
    let url = format!("https://nodejs.org/dist/{ver}/{filename}");
    let tmp = env::temp_dir().join(&filename);
    download(&url, &tmp)?;
    extract_node_archive(&tmp, is_zip)
}

#[cfg(feature = "offline")]
fn provision_node_offline() -> Result<(), String> {
    let bytes = embedded_node_bytes();
    let is_zip = cfg!(target_os = "windows");
    let ext = if is_zip { "zip" } else { "tar.gz" };
    let tmp = env::temp_dir().join(format!("node-asset-offline.{ext}"));
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    extract_node_archive(&tmp, is_zip)
}

#[cfg(feature = "offline")]
fn embedded_node_bytes() -> &'static [u8] {
    #[cfg(target_os = "windows")]
    {
        include_bytes!("../resources/node-asset.zip")
    }
    #[cfg(not(target_os = "windows"))]
    {
        include_bytes!("../resources/node-asset.tar.gz")
    }
}

/// Returns (asset filename for the current target, is_windows_zip).
fn node_asset_name(ver: &str) -> (String, bool) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => (format!("node-{ver}-linux-x64.tar.gz"), false),
        ("linux", "aarch64") => (format!("node-{ver}-linux-arm64.tar.gz"), false),
        ("macos", "x86_64") => (format!("node-{ver}-darwin-x64.tar.gz"), false),
        ("macos", "aarch64") => (format!("node-{ver}-darwin-arm64.tar.gz"), false),
        ("windows", "x86_64") => (format!("node-{ver}-win-x64.zip"), true),
        (os, arch) => (format!("node-{ver}-{arch}-{os}.tar.gz"), false),
    }
}

fn extract_node_archive(tmp: &PathBuf, is_zip: bool) -> Result<(), String> {
    if is_zip {
        // Windows: elevate a PowerShell script that unzips into Program Files.
        let tmpdir = env::temp_dir().join("node-extract-tmp");
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).map_err(|e| e.to_string())?;
        let ps = format!(
            "New-Item -ItemType Directory -Force 'C:\\Program Files\\nodejs' | Out-Null; \
             Expand-Archive -Force '{tmp}' '{tmpdir}'; \
             xcopy /E /Y /H '{tmpdir}\\*' 'C:\\Program Files\\nodejs\\'",
            tmp = tmp.display(),
            tmpdir = tmpdir.display()
        );
        elevate::run_elevated(&format!(
            "powershell -NoProfile -Command \"{ps}\""
        ))
    } else {
        // Unix: extract into /usr/local (bin/node, lib/node_modules, ...).
        let cmd = format!(
            "tar -xzf '{}' -C /usr/local --strip-components=1",
            tmp.display()
        );
        elevate::run_elevated(&cmd)
    }
}

fn download(url: &str, dest: &PathBuf) -> Result<(), String> {
    let resp = reqwest::blocking::get(url).map_err(|e| format!("下载失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载失败，HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    fs::write(dest, &bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn latest_node22() -> Result<String, String> {
    const PINNED: &str = "v22.14.0";
    match reqwest::blocking::get("https://nodejs.org/dist/index.json") {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(v) = resp.json::<serde_json::Value>() {
                if let Some(arr) = v.as_array() {
                    for item in arr {
                        if let Some(ver) = item.get("version").and_then(|x| x.as_str()) {
                            if ver.starts_with("v22") {
                                return Ok(ver.to_string());
                            }
                        }
                    }
                }
            }
            Ok(PINNED.to_string())
        }
        _ => Ok(PINNED.to_string()),
    }
}
