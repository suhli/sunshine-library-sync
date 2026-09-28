use super::{vdf, GameProvider};
use crate::models::*;
use anyhow::{bail, Context, Result};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}};

pub struct SteamProvider { override_path: Option<PathBuf> }
impl SteamProvider { pub fn new(path: Option<PathBuf>) -> Self { Self { override_path: path } } }

pub fn libraries(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::from([root.to_path_buf()]);
    let path = root.join("steamapps/libraryfolders.vdf");
    if path.exists() {
        let parsed = vdf::parse(&fs::read_to_string(&path).with_context(|| format!("Unable to read {}", path.display()))?)?;
        let folders = parsed.get("libraryfolders").or_else(|| parsed.get("LibraryFolders")).and_then(vdf::Value::object).context("libraryfolders.vdf has no libraryfolders object")?;
        for (key, value) in folders {
            if key.parse::<u32>().is_err() { continue; }
            let value = value.text().or_else(|| value.object()?.get("path")?.text());
            if let Some(p) = value { paths.insert(PathBuf::from(p)); }
        }
    }
    Ok(paths.into_iter().collect())
}

pub fn parse_manifest(input: &str, manifest: &Path, library: &Path) -> Result<Option<Game>> {
    let parsed = vdf::parse(input)?;
    let app = parsed.get("AppState").and_then(vdf::Value::object).context("Missing AppState")?;
    let id = vdf::text(app, "appid")?;
    if id.parse::<u64>().is_err() { bail!("Invalid Steam appid"); }
    let flags: u64 = vdf::text(app, "StateFlags")?.parse().context("Invalid StateFlags")?;
    // FullyInstalled (4), even when an update is pending. Transitional installs
    // are not treated as uninstallations by scan_games below.
    if flags & 4 == 0 { bail!("Steam installation is incomplete or changing"); }
    let dir = vdf::text(app, "installdir")?;
    if dir.is_empty() || Path::new(dir).components().any(|x| !matches!(x, std::path::Component::Normal(_))) { bail!("Invalid Steam install directory"); }
    let install = library.join("steamapps/common").join(dir);
    if !install.try_exists()? { return Ok(None); }
    let metadata = ["StateFlags", "LastUpdated", "SizeOnDisk"].iter().filter_map(|k| app.get(*k).and_then(vdf::Value::text).map(|v| (k.to_string(), serde_json::Value::String(v.into())))).collect::<BTreeMap<_, _>>();
    Ok(Some(Game { key: GameKey { provider_id: "steam".into(), provider_game_id: id.into() }, name: vdf::text(app, "name")?.into(), install_path: install, manifest_path: manifest.into(), launch_target: LaunchTarget { uri: format!("steam://rungameid/{id}") }, artwork: None, metadata, sync_status: "new".into() }))
}

impl GameProvider for SteamProvider {
    fn id(&self) -> &'static str { "steam" }
    fn display_name(&self) -> &'static str { "Steam" }
    fn detect(&self) -> Result<ProviderDetection> {
        let mut roots = Vec::new();
        if let Some(p) = &self.override_path { roots.push(p.clone()); }
        else {
            #[cfg(windows)] {
                use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
                for (hive, key, value) in [(HKEY_CURRENT_USER, "Software\\Valve\\Steam", "SteamPath"), (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Valve\\Steam", "InstallPath")] {
                    if let Some(p) = super::registry_string(hive, key, value) { roots.push(p.into()); }
                }
            }
            roots.push(PathBuf::from(std::env::var_os("ProgramFiles(x86)").unwrap_or_else(|| "C:\\Program Files (x86)".into())).join("Steam"));
        }
        let fallback = roots.first().cloned();
        for root in roots {
            if root.join("steamapps").try_exists()? {
                let data_paths = libraries(&root)?.into_iter().map(|p| p.join("steamapps")).collect();
                return Ok(ProviderDetection { installed: true, launcher_path: Some(root), data_paths, version: None });
            }
        }
        Ok(ProviderDetection { launcher_path: fallback.clone(), data_paths: fallback.map(|r| vec![r.join("steamapps")]).unwrap_or_default(), ..Default::default() })
    }
    fn scan_games(&self, detection: &ProviderDetection) -> Result<ProviderScan> {
        let mut result = ProviderScan { complete: true, ..Default::default() };
        for directory in &detection.data_paths {
            let entries = match fs::read_dir(directory) { Ok(x) => x, Err(e) => { result.complete = false; result.warnings.push(format!("Unable to read library {}: {e}", directory.display())); continue; } };
            for item in entries {
                let item = item?; let path = item.path();
                if !path.file_name().and_then(|x| x.to_str()).is_some_and(|s| s.starts_with("appmanifest_") && s.ends_with(".acf")) { continue; }
                match fs::read_to_string(&path).map_err(anyhow::Error::from).and_then(|s| parse_manifest(&s, &path, directory.parent().context("Invalid Steam library path")?)) {
                    Ok(Some(mut game)) => { game.launch_target = self.launch_command(&game)?; result.games.push(game); },
                    Ok(None) => {},
                    Err(e) => { result.complete = false; result.warnings.push(format!("{}: {e}", path.display())); }
                }
            }
        }
        Ok(result)
    }
    fn launch_command(&self, game: &Game) -> Result<LaunchTarget> {
        let id: u64 = game.key.provider_game_id.parse()?;
        Ok(LaunchTarget { uri: format!("steam://rungameid/{id}") })
    }
    fn artwork_candidates(&self, game: &Game) -> Vec<ArtworkCandidate> {
        let mut out = Vec::new();
        if let Ok(d) = self.detect() { if let Some(root) = d.launcher_path {
            let id = &game.key.provider_game_id;
            for p in [root.join(format!("appcache/librarycache/{id}_library_600x900.jpg")), root.join(format!("appcache/librarycache/{id}/library_600x900.jpg"))] { out.push(ArtworkCandidate::Local(p)); }
            // Recent Steam builds put hashed artwork names under an appid directory.
            if let Ok(entries) = fs::read_dir(root.join(format!("appcache/librarycache/{id}"))) {
                for entry in entries.flatten() { let n = entry.file_name().to_string_lossy().to_lowercase(); if n.contains("library_600x900") { out.push(ArtworkCandidate::Local(entry.path())); } }
            }
        } }
        out.push(ArtworkCandidate::Remote(format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg", game.key.provider_game_id)));
        out
    }
}
