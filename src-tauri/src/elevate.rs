use std::process::Command;

/// Returns true when the current process already has administrator/root rights.
pub fn is_root() -> bool {
    #[cfg(unix)]
    unsafe {
        libc::geteuid() == 0
    }
    #[cfg(windows)]
    {
        false
    }
}

/// Run a platform-native shell command, elevating (UAC / sudo / pkexec) when needed.
/// On Unix `cmd` is a `sh -c` string; on Windows it is a `cmd /C` string
/// (typically wrapping a PowerShell call).
pub fn run_elevated(cmd: &str) -> Result<(), String> {
    if is_root() {
        run_shell(cmd)
    } else {
        run_shell_elevated(cmd)
    }
}

fn run_shell(cmd: &str) -> Result<(), String> {
    let (shell, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
    let status = Command::new(shell)
        .arg(flag)
        .arg(cmd)
        .status()
        .map_err(|e| format!("执行命令失败: {e}"))?;
    status_to_result(status, cmd)
}

#[cfg(target_os = "windows")]
fn run_shell_elevated(cmd: &str) -> Result<(), String> {
    // Spawn an elevated cmd.exe that runs our command.
    let inner = cmd.replace('\'', "''");
    let ps = format!(
        "Start-Process -Verb RunAs -FilePath cmd.exe -ArgumentList '/C', '{inner}'"
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .status()
        .map(|s| status_to_result(s, cmd))
        .unwrap_or_else(|e| Err(e.to_string()))
}

#[cfg(target_os = "macos")]
fn run_shell_elevated(cmd: &str) -> Result<(), String> {
    let osa = format!(
        "do shell script \"{}\" with administrator privileges",
        cmd.replace('"', "\\\"")
    );
    Command::new("osascript")
        .args(["-e", &osa])
        .status()
        .map(|s| status_to_result(s, cmd))
        .unwrap_or_else(|e| Err(e.to_string()))
}

#[cfg(target_os = "linux")]
fn run_shell_elevated(cmd: &str) -> Result<(), String> {
    if let Ok(s) = Command::new("pkexec").args(["sh", "-c", cmd]).status() {
        if s.success() {
            return Ok(());
        }
    }
    Command::new("sudo")
        .args(["sh", "-c", cmd])
        .status()
        .map(|s| status_to_result(s, cmd))
        .unwrap_or_else(|e| Err(e.to_string()))
}

fn status_to_result(status: std::process::ExitStatus, cmd: &str) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "命令执行失败（退出码 {}）: {}",
            status.code().unwrap_or(-1),
            cmd
        ))
    }
}
