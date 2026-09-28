use super::GameProvider;
use crate::models::*;
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub struct EpicProvider {
    override_path: Option<PathBuf>,
}
impl EpicProvider {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            override_path: path,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub app_name: String,
    pub display_name: String,
    pub install_location: PathBuf,
    #[serde(default)]
    pub launch_executable: String,
    #[serde(default)]
    pub catalog_namespace: String,
    #[serde(default)]
    pub catalog_item_id: String,
    #[serde(default)]
    pub app_categories: Vec<String>,
    #[serde(rename = "bIsIncompleteInstall", default)]
    pub incomplete: bool,
    #[serde(rename = "bIsApplication", default)]
    pub is_application: Option<bool>,
    #[serde(rename = "bIsExecutable", default)]
    pub is_executable: Option<bool>,
    #[serde(default)]
    pub main_game_app_name: String,
    #[serde(default)]
    pub display_assets: Vec<serde_json::Value>,
}
pub fn encode_component(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
pub fn parse_manifest(input: &str, path: &Path) -> Result<Option<Game>> {
    let m: Manifest = serde_json::from_str(input.trim_start_matches('\u{feff}'))
        .context("Invalid Epic manifest")?;
    if m.incomplete {
        bail!("Epic installation is incomplete or changing");
    }
    let categories: Vec<_> = m.app_categories.iter().map(|s| s.to_lowercase()).collect();
    let engine = m.app_name.starts_with("UE_")
        || categories.iter().any(|s| {
            [
                "engines",
                "engine",
                "plugins",
                "plugin",
                "mods",
                "editor",
                "templates",
            ]
            .contains(&s.as_str())
        });
    let addon = categories
        .iter()
        .any(|s| ["dlc", "addons", "addon"].contains(&s.as_str()))
        || (!m.main_game_app_name.is_empty() && m.main_game_app_name != m.app_name);
    let launchable_addon = m.is_executable == Some(true) && m.is_application == Some(true);
    if engine
        || (addon && !launchable_addon)
        || m.is_application == Some(false)
        || m.launch_executable.is_empty()
        || !m.install_location.try_exists()?
    {
        return Ok(None);
    }
    if m.app_name.is_empty() || m.display_name.is_empty() {
        bail!("Epic manifest has an empty ID or name");
    }
    let target = if m.catalog_namespace.is_empty() || m.catalog_item_id.is_empty() {
        encode_component(&m.app_name)
    } else {
        encode_component(&format!(
            "{}:{}:{}",
            m.catalog_namespace, m.catalog_item_id, m.app_name
        ))
    };
    let mut metadata = BTreeMap::new();
    metadata.insert("catalog_namespace".into(), m.catalog_namespace.into());
    metadata.insert("catalog_item_id".into(), m.catalog_item_id.into());
    metadata.insert("launch_executable".into(), m.launch_executable.into());
    metadata.insert("display_assets".into(), m.display_assets.into());
    Ok(Some(Game {
        key: GameKey {
            provider_id: "epic".into(),
            provider_game_id: m.app_name,
        },
        name: m.display_name,
        install_path: m.install_location,
        manifest_path: path.into(),
        launch_target: LaunchTarget {
            uri: format!("com.epicgames.launcher://apps/{target}?action=launch&silent=true"),
        },
        artwork: None,
        metadata,
        sync_status: "new".into(),
    }))
}
impl GameProvider for EpicProvider {
    fn id(&self) -> &'static str {
        "epic"
    }
    fn display_name(&self) -> &'static str {
        "Epic Games"
    }
    fn detect(&self) -> Result<ProviderDetection> {
        let data = self.override_path.clone().unwrap_or_else(|| {
            PathBuf::from(
                std::env::var_os("ProgramData").unwrap_or_else(|| "C:\\ProgramData".into()),
            )
            .join("Epic/EpicGamesLauncher/Data/Manifests")
        });
        #[cfg(windows)]
        let launcher = super::registry_string(
            winreg::enums::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher",
            "InstallLocation",
        )
        .map(PathBuf::from);
        #[cfg(not(windows))]
        let launcher = None;
        let default_launcher = PathBuf::from(
            std::env::var_os("ProgramFiles(x86)")
                .unwrap_or_else(|| "C:\\Program Files (x86)".into()),
        )
        .join("Epic Games/Launcher");
        let launcher = launcher.or_else(|| default_launcher.is_dir().then_some(default_launcher));
        Ok(ProviderDetection {
            installed: data.try_exists()? || launcher.is_some(),
            launcher_path: launcher,
            data_paths: vec![data],
            version: None,
        })
    }
    fn scan_games(&self, detection: &ProviderDetection) -> Result<ProviderScan> {
        let mut result = ProviderScan {
            complete: true,
            ..Default::default()
        };
        for directory in &detection.data_paths {
            for entry in fs::read_dir(directory).with_context(|| {
                format!(
                    "Unable to read Epic manifest directory {}",
                    directory.display()
                )
            })? {
                let path = entry?.path();
                if path.extension().and_then(|s| s.to_str()) != Some("item") {
                    continue;
                }
                match fs::read_to_string(&path)
                    .map_err(anyhow::Error::from)
                    .and_then(|s| parse_manifest(&s, &path))
                {
                    Ok(Some(game)) => result.games.push(game),
                    Ok(None) => {}
                    Err(e) => {
                        result.complete = false;
                        result.warnings.push(format!("{}: {e}", path.display()));
                    }
                }
            }
        }
        Ok(result)
    }
    fn launch_command(&self, game: &Game) -> Result<LaunchTarget> {
        Ok(game.launch_target.clone())
    }
    fn artwork_candidates(&self, game: &Game) -> Vec<ArtworkCandidate> {
        // Structured local asset references, when present, can be used without
        // touching account data or attempting to decode Chromium HTTP caches.
        game.metadata
            .get("display_assets")
            .and_then(|a| a.as_array())
            .into_iter()
            .flatten()
            .filter_map(|a| a.get("LocalPath").or_else(|| a.get("Path"))?.as_str())
            .map(|s| ArtworkCandidate::Local(PathBuf::from(s)))
            .collect()
    }
}
