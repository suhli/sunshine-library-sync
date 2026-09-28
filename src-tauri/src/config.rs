use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub general: General,
    pub providers: BTreeMap<String, ProviderSettings>,
    pub excluded_games: BTreeSet<String>,
    pub sunshine: SunshineSettings,
    pub network: NetworkSettings,
    pub artwork: ArtworkSettings,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    pub auto_sync: bool,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub close_behavior: String,
    pub theme: String,
    pub language: String,
}
impl Default for General {
    fn default() -> Self {
        Self {
            auto_sync: false,
            start_with_windows: false,
            start_minimized: false,
            close_behavior: "tray".into(),
            theme: "system".into(),
            language: "system".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderSettings {
    pub enabled: bool,
    pub path: Option<PathBuf>,
}
impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SunshineSettings {
    pub install_path: Option<PathBuf>,
    pub apps_path: Option<PathBuf>,
    pub reload_mode: String,
    pub reload_command: String,
}
impl Default for SunshineSettings {
    fn default() -> Self {
        Self {
            install_path: None,
            apps_path: None,
            reload_mode: "none".into(),
            reload_command: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    pub proxy_mode: String,
    pub proxy_url: String,
}
impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            proxy_mode: "system".into(),
            proxy_url: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ArtworkSettings {
    pub provider: String,
    pub steamgriddb_api_key: String,
}
impl Default for ArtworkSettings {
    fn default() -> Self {
        Self {
            provider: "auto".into(),
            steamgriddb_api_key: String::new(),
        }
    }
}

impl Settings {
    pub fn provider(&self, id: &str) -> ProviderSettings {
        self.providers.get(id).cloned().unwrap_or_default()
    }
    pub fn validate(&self) -> Result<()> {
        if !["tray", "exit"].contains(&self.general.close_behavior.as_str()) {
            bail!("Invalid close behavior");
        }
        if !["system", "light", "dark"].contains(&self.general.theme.as_str()) {
            bail!("Invalid theme");
        }
        if !["system", "en", "zh-CN"].contains(&self.general.language.as_str()) {
            bail!("Invalid language");
        }
        if !["none", "restart", "command"].contains(&self.sunshine.reload_mode.as_str()) {
            bail!("Invalid Sunshine reload mode");
        }
        if self.sunshine.reload_mode == "command" && self.sunshine.reload_command.trim().is_empty()
        {
            bail!("Enter a custom reload command");
        }
        if !["system", "direct", "custom"].contains(&self.network.proxy_mode.as_str()) {
            bail!("Invalid proxy mode");
        }
        if self.network.proxy_mode == "custom" {
            let u = url::Url::parse(&self.network.proxy_url).context("Enter a valid proxy URL")?;
            if !["http", "https", "socks5", "socks5h"].contains(&u.scheme())
                || u.host_str().is_none()
            {
                bail!("Proxy must use HTTP, HTTPS, or SOCKS5");
            }
        }
        if !["auto", "steam", "epic", "steamgriddb", "local"]
            .contains(&self.artwork.provider.as_str())
        {
            bail!("Invalid artwork source");
        }
        Ok(())
    }
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let s: Self = toml::from_str(&fs::read_to_string(path)?)
            .context("config.toml is invalid; original file was preserved")?;
        s.validate()?;
        Ok(s)
    }
    pub fn save(&self, root: &Path) -> Result<()> {
        self.validate()?;
        crate::storage::atomic_write(
            &root.join("config.toml"),
            toml::to_string_pretty(self)?.as_bytes(),
        )
    }
}
pub fn data_root() -> PathBuf {
    data_root_for(
        cfg!(debug_assertions),
        std::env::var_os("SUNSHINE_LIBRARY_SYNC_DATA"),
        std::env::var_os("LOCALAPPDATA"),
    )
}

fn data_root_for(
    debug_build: bool,
    fixture_override: Option<std::ffi::OsString>,
    local_app_data: Option<std::ffi::OsString>,
) -> PathBuf {
    if debug_build {
        if let Some(path) = fixture_override {
            return PathBuf::from(path);
        }
    }
    PathBuf::from(local_app_data.unwrap_or_else(|| ".".into())).join("SunshineLibrarySync")
}

#[cfg(test)]
mod tests {
    use super::{data_root_for, Settings};

    #[test]
    fn fixture_override_is_only_used_by_debug_builds() {
        let fixture = Some("test-fixture".into());
        let local = Some("user-data".into());
        assert_eq!(
            data_root_for(true, fixture.clone(), local.clone()),
            std::path::PathBuf::from("test-fixture")
        );
        assert_eq!(
            data_root_for(false, fixture, local),
            std::path::PathBuf::from("user-data").join("SunshineLibrarySync")
        );
    }

    #[test]
    fn previous_settings_follow_system_language_and_invalid_language_is_rejected() {
        let mut document = toml::Value::try_from(Settings::default()).unwrap();
        document["general"]
            .as_table_mut()
            .unwrap()
            .remove("language");
        let migrated: Settings = document.clone().try_into().unwrap();
        assert_eq!(migrated.general.language, "system");
        document["general"]
            .as_table_mut()
            .unwrap()
            .insert("language".into(), "unsupported".into());
        let invalid: Settings = document.try_into().unwrap();
        assert!(invalid.validate().is_err());
    }
}
