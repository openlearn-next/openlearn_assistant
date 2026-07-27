use crate::settings;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Delete user data (~/openlearn-next) and logs.
pub fn clean_data() -> Result<(), String> {
    let dir = settings::home_dir().join("openlearn-next");
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_file(settings::log_path());
    let _ = fs::remove_file(settings::pid_path());
    Ok(())
}

/// Query the latest openlearn-next version from the npm registry.
pub fn latest_version() -> Option<String> {
    let out = Command::new("npm")
        .args(["view", "openlearn-next", "version"])
        .output()
        .ok()?;
    if out.status.success() {
        let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !ver.is_empty() {
            return Some(ver);
        }
    }
    None
}

/// Fetch version list from npm registry, returning the most recent entries.
/// `offset` skips the N most recent versions; `limit` caps the returned count.
pub fn list_versions(offset: usize, limit: usize) -> Result<Vec<String>, String> {
    let out = Command::new("npm")
        .args(["view", "openlearn-next", "versions", "--json"])
        .output()
        .map_err(|e| format!("npm view 失败: {e}"))?;
    if !out.status.success() {
        return Err("npm view 退出码非零".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut versions: Vec<String> = serde_json::from_str(&text)
        .map_err(|e| format!("解析版本列表失败: {e}"))?;
    // npm returns versions in chronological order; reverse for newest-first.
    versions.reverse();
    let end = (offset + limit).min(versions.len());
    Ok(versions[offset.min(versions.len())..end].to_vec())
}

/// Cached package directory for a given version.
fn pkg_cache_dir(version: &str) -> PathBuf {
    settings::app_dir().join("pkg").join(version)
}

/// Ensure the openlearn-next package for `version` is downloaded, patched,
/// and its dependencies installed under the local cache. Returns the cache
/// directory path so the caller can invoke the binary directly.
pub fn ensure_package(version: &str, mirror: bool) -> Result<PathBuf, String> {
    let cache = pkg_cache_dir(version);
    if cache.join("node_modules").exists() {
        return Ok(cache);
    }

    // Clean any partial state
    let _ = fs::remove_dir_all(&cache);
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;

    let registry_flag = if mirror {
        vec!["--registry=https://registry.npmmirror.com".to_string()]
    } else {
        vec![]
    };

    // 1. npm pack to tempdir
    let tmpdir = env::temp_dir().join(format!("openlearn-pkg-{}", version));
    let _ = fs::remove_dir_all(&tmpdir);
    fs::create_dir_all(&tmpdir).map_err(|e| e.to_string())?;

    let pkg_spec = if version == "latest" || version.is_empty() {
        "openlearn-next".to_string()
    } else {
        format!("openlearn-next@{}", version)
    };

    let mut pack_cmd = Command::new("npm");
    for flag in &registry_flag {
        pack_cmd.arg(flag);
    }
    let status = pack_cmd
        .args(["pack", &pkg_spec, "--pack-destination"])
        .arg(&tmpdir)
        .status()
        .map_err(|e| format!("npm pack 失败: {e}"))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&tmpdir);
        return Err(format!("npm pack 退出码: {}", status.code().unwrap_or(-1)));
    }

    // 2. Find tarball
    let tgz = find_tarball(&tmpdir)?;

    // 3. Extract to cache
    let status = Command::new("tar")
        .args(["-xzf", &tgz.to_string_lossy(), "-C", &cache.to_string_lossy(), "--strip-components=1"])
        .status()
        .map_err(|e| format!("解压失败: {e}"))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&cache);
        return Err(format!("解压退出码: {}", status.code().unwrap_or(-1)));
    }

    // Clean tempdir
    let _ = fs::remove_dir_all(&tmpdir);

    // 4. Patch workspace:* deps
    let pkg_json = cache.join("package.json");
    let raw = fs::read_to_string(&pkg_json).map_err(|e| e.to_string())?;
    let patched = resolve_workspace_deps(&raw)?;
    fs::write(&pkg_json, patched).map_err(|e| e.to_string())?;

    // 5. npm install --omit=dev
    let mut install_cmd = Command::new("npm");
    for flag in &registry_flag {
        install_cmd.arg(flag);
    }
    let status = install_cmd
        .current_dir(&cache)
        .args(["install", "--omit=dev"])
        .status()
        .map_err(|e| format!("npm install 失败: {e}"))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&cache);
        return Err(format!("npm install 退出码: {}", status.code().unwrap_or(-1)));
    }

    Ok(cache)
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
