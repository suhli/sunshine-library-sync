use anyhow::{bail, Context, Result};
use std::{
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

pub fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}
#[cfg(windows)]
pub fn detect_service() -> Option<String> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let services = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SYSTEM\\CurrentControlSet\\Services")
        .ok()?;
    services.enum_keys().flatten().find(|name| {
        services
            .open_subkey(name)
            .ok()
            .and_then(|k| k.get_value::<String, _>("ImagePath").ok())
            .is_some_and(|path| {
                let lower = path.to_lowercase();
                lower.contains("\\sunshinesvc.exe") || lower.contains("\\sunshine.exe")
            })
    })
}
#[cfg(not(windows))]
pub fn detect_service() -> Option<String> {
    None
}
pub fn query(name: &str) -> Result<String> {
    let out = hidden_command("sc.exe").args(["query", name]).output()?;
    if !out.status.success() {
        bail!("Unable to query Sunshine service");
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(if text.contains("RUNNING") {
        "running"
    } else if text.contains("STOPPED") {
        "stopped"
    } else {
        "pending"
    }
    .into())
}
pub fn restart() -> Result<()> {
    let service = detect_service().context("Sunshine Windows service was not detected")?;
    let state = query(&service)?;
    if state != "stopped" {
        let out = hidden_command("sc.exe").args(["stop", &service]).output()?;
        if !out.status.success() {
            bail!("Sunshine service stop failed. Administrator rights may be required.");
        }
        let deadline = Instant::now() + Duration::from_secs(25);
        while query(&service)? != "stopped" {
            if Instant::now() > deadline {
                bail!("Timed out waiting for Sunshine service to stop");
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    let out = hidden_command("sc.exe")
        .args(["start", &service])
        .output()?;
    if !out.status.success() {
        bail!("Sunshine service start failed. Administrator rights may be required.");
    }
    let deadline = Instant::now() + Duration::from_secs(25);
    while query(&service)? != "running" {
        if Instant::now() > deadline {
            bail!("Timed out waiting for Sunshine service to start");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}
pub fn file_version(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let literal = path.to_string_lossy().replace('\'', "''");
    let script =
        format!("[System.Diagnostics.FileVersionInfo]::GetVersionInfo('{literal}').ProductVersion");
    let output = hidden_command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!version.is_empty()).then_some(version)
}
pub fn reload(config: &crate::config::SunshineSettings) -> Result<()> {
    match config.reload_mode.as_str() {
        "none" => Ok(()),
        "restart" => restart(),
        "command" => {
            // This command is explicitly supplied by the user in Settings.
            let mut child = hidden_command("cmd.exe")
                .args(["/D", "/S", "/C", &config.reload_command])
                .spawn()?;
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                if let Some(status) = child.try_wait()? {
                    if status.success() {
                        return Ok(());
                    }
                    bail!("Custom reload command failed");
                }
                if Instant::now() > deadline {
                    let _ = hidden_command("taskkill.exe")
                        .args(["/PID", &child.id().to_string(), "/T", "/F"])
                        .status();
                    let _ = child.wait();
                    bail!("Custom reload command timed out");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        _ => bail!("Invalid reload mode"),
    }
}
