use crate::node;
use crate::pkg;
use crate::settings;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize)]
pub struct Status {
    pub running: bool,
    pub pid: Option<u32>,
    pub port: u16,
    pub version: Option<String>,
    /// Node >= 22 present (required by better-sqlite3).
    pub node_ok: bool,
}

pub fn status() -> Status {
    let s = settings::load_settings();
    let running = is_running();
    let pid = read_pid();
    let version = pkg::latest_version();
    let node_ok = node::detect_node().meets_requirement;
    Status {
        running,
        pid,
        port: s.port,
        version,
        node_ok,
    }
}

pub fn start_service() -> Result<(), String> {
    if is_running() {
        return Err("openlearn-next 已在运行".into());
    }
    let s = settings::load_settings();
    fs::create_dir_all(settings::app_dir()).map_err(|e| e.to_string())?;

    // Ensure package is cached locally (download + patch + install deps).
    let cache = pkg::ensure_package(&s.version, s.mirror_enabled)?;
    let entry = resolve_bin_entry(&cache)?;

    let log = fs::File::create(settings::log_path()).map_err(|e| e.to_string())?;
    let log_err = log.try_clone().map_err(|e| e.to_string())?;

    let mut cmd = Command::new("node");
    cmd.arg(&entry);
    cmd.arg("-p").arg(s.port.to_string());
    cmd.env("OPENLEARN_DB_PATH", &s.db_path);
    if !s.gemini_api_key.is_empty() {
        cmd.env("GEMINI_API_KEY", &s.gemini_api_key);
    }
    // Detach into its own session/process group so it survives the GUI exiting.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0000_0008); // DETACHED_PROCESS
    }
    cmd.stdout(Stdio::from(log));
    cmd.stderr(Stdio::from(log_err));

    let child = cmd.spawn().map_err(|e| format!("启动失败: {e}"))?;
    let pid = child.id();
    fs::write(settings::pid_path(), pid.to_string()).map_err(|e| e.to_string())?;
    // Drop Child: the OS process keeps running independently.
    drop(child);

    health_check(s.port);
    Ok(())
}

pub fn stop_service() -> Result<(), String> {
    let pid = read_pid().ok_or("openlearn-next 未在运行（无 PID 文件）")?;
    kill_process(pid)?;
    let _ = fs::remove_file(settings::pid_path());
    Ok(())
}

pub fn is_running() -> bool {
    read_pid().map(process_alive).unwrap_or(false)
}

pub fn get_logs(tail: u32) -> String {
    match fs::read_to_string(settings::log_path()) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let n = tail.max(1) as usize;
            let start = lines.len().saturating_sub(n);
            lines[start..].join("\n")
        }
        Err(_) => String::new(),
    }
}

/// Read package.json bin field and resolve the entry script path.
fn resolve_bin_entry(cache: &Path) -> Result<std::path::PathBuf, String> {
    let pkg_json = cache.join("package.json");
    let raw = fs::read_to_string(&pkg_json)
        .map_err(|e| format!("读取 package.json 失败: {e}"))?;
    let root: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("解析 package.json 失败: {e}"))?;

    let bin_val = root["bin"].as_object().ok_or("package.json 缺少 bin 字段")?;
    let script = bin_val["openlearn-next"]
        .as_str()
        .or_else(|| bin_val.values().next().and_then(|v| v.as_str()))
        .ok_or("无法确定可执行入口")?;

    let entry = cache.join(script);
    if !entry.exists() {
        return Err(format!("找不到可执行文件: {}", entry.display()));
    }
    Ok(entry)
}

fn read_pid() -> Option<u32> {
    fs::read_to_string(settings::pid_path())
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}")])
        .output();
    out.map(|o| {
        let text = String::from_utf8_lossy(&o.stdout);
        text.contains(&format!("PID eq {pid}")) || text.contains(&pid.to_string())
    })
    .unwrap_or(false)
}

#[cfg(unix)]
fn kill_process(pid: u32) -> Result<(), String> {
    // Kill the whole process group (negative pid) so Worker Threads die too.
    let _ = Command::new("kill")
        .args(["-TERM", &format!("-{}", pid)])
        .status();
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    Ok(())
}

#[cfg(windows)]
fn kill_process(pid: u32) -> Result<(), String> {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn health_check(port: u16) {
    let url = format!("http://127.0.0.1:{port}/");
    for _ in 0..120 {
        if let Ok(resp) = reqwest::blocking::get(&url) {
            if resp.status().is_success() {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
