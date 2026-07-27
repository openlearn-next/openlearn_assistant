use crate::service;
use crate::settings;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

fn pm_cmd(subcommand: &str) -> Command {
    let pm = pkg_manager();
    let mut cmd = Command::new(pm);
    if pm == "npm" {
        cmd.arg("--prefix").arg(npm_user_prefix());
    }
    cmd.arg(subcommand);
    cmd
}

/// Persistent directory where we extract and install openlearn-next locally before
/// linking globally. This avoids pnpm tarball-install issues.
fn pkg_install_dir() -> PathBuf {
    settings::app_dir().join("openlearn-next-package")
}

pub fn npm_global_bin() -> Option<String> {
    let out = pm_cmd("bin").arg("-g").output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

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
    let _ = run_pm_global("remove", &["openlearn-next"]);
    #[cfg(feature = "offline")]
    {
        let tgz = write_embedded_openlearn()?;
        let dir = extract_and_patch(&tgz, &pkg_install_dir())?;
        install_deps_and_link(&dir)
    }
    #[cfg(not(feature = "offline"))]
    {
        let tgz = download_openlearn_tarball()?;
        let dir = extract_and_patch(&tgz, &pkg_install_dir())?;
        install_deps_and_link(&dir)
    }
}

pub fn uninstall_pkg(keep_data: bool) -> Result<(), String> {
    run_pm_global("remove", &["openlearn-next"])?;
    let _ = fs::remove_dir_all(pkg_install_dir());
    if !keep_data {
        let dir = settings::home_dir().join("openlearn-next");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(settings::log_path());
    }
    Ok(())
}

pub fn upgrade_pkg() -> Result<(), String> {
    let _ = service::stop_service();
    let _ = run_pm_global("remove", &["openlearn-next"]);
    let _ = fs::remove_dir_all(pkg_install_dir());
    let tgz = download_openlearn_tarball()?;
    let dir = extract_and_patch(&tgz, &pkg_install_dir())?;
    install_deps_and_link(&dir)?;
    service::start_service()?;
    Ok(())
}

/// Download openlearn-next tarball from npm registry.
fn download_openlearn_tarball() -> Result<PathBuf, String> {
    let tmpdir = env::temp_dir().join("openlearn-install");
    let _ = fs::remove_dir_all(&tmpdir);
    fs::create_dir_all(&tmpdir).map_err(|e| e.to_string())?;

    let status = Command::new("npm")
        .args(["pack", "openlearn-next", "--pack-destination"])
        .arg(&tmpdir)
        .status()
        .map_err(|e| format!("npm pack 失败: {e}"))?;
    if !status.success() {
        return Err(format!("npm pack 退出码: {}", status.code().unwrap_or(-1)));
    }

    find_tarball(&tmpdir)
}

/// Extract tarball into dest, patch workspace:* deps.
fn extract_and_patch(tarball: &PathBuf, dest: &PathBuf) -> Result<PathBuf, String> {
    let _ = fs::remove_dir_all(dest);
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;

    let status = Command::new("tar")
        .args(["-xzf", &tarball.to_string_lossy(), "-C", &dest.to_string_lossy(), "--strip-components=1"])
        .status()
        .map_err(|e| format!("解压失败: {e}"))?;
    if !status.success() {
        return Err(format!("解压退出码: {}", status.code().unwrap_or(-1)));
    }

    // Clean temp dir
    if let Some(parent) = tarball.parent() {
        if parent.starts_with(env::temp_dir()) {
            let _ = fs::remove_dir_all(parent);
        }
    }

    let pkg_json = dest.join("package.json");
    let raw = fs::read_to_string(&pkg_json).map_err(|e| e.to_string())?;
    let patched = resolve_workspace_deps(&raw)?;
    fs::write(&pkg_json, patched).map_err(|e| e.to_string())?;

    Ok(dest.clone())
}

fn find_tarball(dir: &PathBuf) -> Result<PathBuf, String> {
    fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name().to_string_lossy().starts_with("openlearn-next-")
                && e.file_name().to_string_lossy().ends_with(".tgz")
        })
        .map(|e| e.path())
        .ok_or("找不到 openlearn-next tarball".into())
}

/// Run `pnpm install --prod` locally, then `pnpm link --global` to register
/// as a global package with all dependencies resolved.
fn install_deps_and_link(dir: &PathBuf) -> Result<(), String> {
    let pm = pkg_manager();
    let install_cmd = if pm == "pnpm" { "install" } else { "install" };
    let flag = if pm == "pnpm" { "--prod" } else { "--omit=dev" };

    let status = Command::new(pm)
        .current_dir(dir)
        .args([install_cmd, flag])
        .status()
        .map_err(|e| format!("安装依赖失败: {e}"))?;
    if !status.success() {
        return Err(format!("安装依赖退出码: {}", status.code().unwrap_or(-1)));
    }

    // Link globally
    let status = Command::new(pm)
        .current_dir(dir)
        .args(["link", "--global"])
        .status()
        .map_err(|e| format!("全局链接失败: {e}"))?;
    if !status.success() {
        return Err(format!("全局链接退出码: {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}

fn resolve_workspace_deps(json: &str) -> Result<String, String> {
    let mut root: Value = serde_json::from_str(json).map_err(|e| format!("解析 package.json 失败: {e}"))?;

    for key in &["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = root.get_mut(*key).and_then(|d| d.as_object_mut()) {
            for (name, version) in deps.iter_mut() {
                if let Some(v) = version.as_str() {
                    if v.starts_with("workspace:") {
                        let resolved = lookup_npm_version(name)?;
                        *version = Value::String(resolved);
                    }
                }
            }
        }
    }

    serde_json::to_string_pretty(&root).map_err(|e| format!("序列化 package.json 失败: {e}"))
}

fn lookup_npm_version(pkg: &str) -> Result<String, String> {
    let out = Command::new("npm")
        .args(["view", pkg, "version"])
        .output()
        .map_err(|e| format!("npm view {} 失败: {}", pkg, e))?;
    if out.status.success() {
        let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !ver.is_empty() {
            return Ok(format!("^{}", ver));
        }
    }
    Ok("*".to_string())
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
