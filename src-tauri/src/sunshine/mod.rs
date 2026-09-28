pub mod service;
use crate::{config::SunshineSettings, models::SunshineStatus};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn apps_path(config: &SunshineSettings) -> Result<PathBuf> {
    if let Some(path) = &config.apps_path {
        if !path.as_os_str().is_empty() {
            return Ok(path.clone());
        }
    }
    let root = install_path(config);
    let conf_dir = root.join("config");
    let conf = conf_dir.join("sunshine.conf");
    if conf.try_exists()? {
        let text = fs::read_to_string(conf).context("Unable to read sunshine.conf")?;
        if let Some(value) = file_apps(&text) {
            let p = PathBuf::from(value);
            return Ok(if p.is_absolute() { p } else { conf_dir.join(p) });
        }
    }
    Ok(conf_dir.join("apps.json"))
}
pub fn file_apps(input: &str) -> Option<String> {
    input
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            (key.trim() == "file_apps")
                .then(|| value.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
        })
        .next_back()
}
pub fn install_path(config: &SunshineSettings) -> PathBuf {
    config
        .install_path
        .clone()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| {
            PathBuf::from(
                std::env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\Program Files".into()),
            )
            .join("Sunshine")
        })
}
pub fn read_apps(path: &Path) -> Result<(Vec<u8>, Value)> {
    let bytes = fs::read(path).with_context(|| format!("Unable to read Sunshine apps.json at {}. Locate an existing configuration and check permissions.", path.display()))?;
    let content = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let json: Value = serde_json::from_slice(content)
        .context("Sunshine apps.json is invalid; original file was preserved")?;
    let apps = json
        .get("apps")
        .and_then(Value::as_array)
        .context("Sunshine apps.json must contain an apps array")?;
    if apps
        .iter()
        .any(|a| !a.is_object() || !a.get("name").is_some_and(Value::is_string))
    {
        bail!("Sunshine apps.json contains an invalid application; original file was preserved");
    }
    Ok((bytes, json))
}
pub fn status(config: &SunshineSettings, managed: usize) -> SunshineStatus {
    let root = install_path(config);
    let mut status = SunshineStatus {
        detected: root.join("sunshine.exe").exists(),
        install_path: Some(root),
        managed,
        service_status: "not-installed".into(),
        ..Default::default()
    };
    match apps_path(config) {
        Ok(path) => {
            if path.exists() {
                status.detected = true;
                match read_apps(&path) {
                    Ok((_, json)) => {
                        status.applications = json["apps"].as_array().map_or(0, Vec::len)
                    }
                    Err(e) => status.error = Some(e.to_string()),
                }
            }
            status.apps_path = Some(path);
        }
        Err(e) => status.error = Some(e.to_string()),
    }
    if let Some(name) = service::detect_service() {
        status.service_status = service::query(&name).unwrap_or_else(|_| "unknown".into());
        status.service_name = Some(name);
    }
    status.version = status
        .install_path
        .as_ref()
        .and_then(|root| service::file_version(&root.join("sunshine.exe")));
    status
}
