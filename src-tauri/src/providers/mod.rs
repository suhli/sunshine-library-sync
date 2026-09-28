pub mod steam;
pub mod epic;
pub mod vdf;
use crate::{config::Settings, models::*};
use anyhow::Result;
use std::path::PathBuf;

pub trait GameProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn detect(&self) -> Result<ProviderDetection>;
    fn scan_games(&self, detection: &ProviderDetection) -> Result<ProviderScan>;
    fn watch_paths(&self, detection: &ProviderDetection) -> Vec<PathBuf> { detection.data_paths.clone() }
    fn launch_command(&self, game: &Game) -> Result<LaunchTarget>;
    fn artwork_candidates(&self, game: &Game) -> Vec<ArtworkCandidate>;
}

pub fn registry(settings: &Settings) -> Vec<Box<dyn GameProvider>> {
    vec![Box::new(steam::SteamProvider::new(settings.provider("steam").path)), Box::new(epic::EpicProvider::new(settings.provider("epic").path))]
}

#[cfg(windows)]
pub fn registry_string(root: winreg::HKEY, key: &str, value: &str) -> Option<String> {
    winreg::RegKey::predef(root).open_subkey(key).ok()?.get_value::<String, _>(value).ok()
}
