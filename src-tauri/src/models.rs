use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GameKey { pub provider_id: String, pub provider_game_id: String }
impl GameKey { pub fn encoded(&self) -> String { format!("{}:{}", self.provider_id, self.provider_game_id) } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchTarget { pub uri: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub key: GameKey,
    pub name: String,
    pub install_path: PathBuf,
    pub manifest_path: PathBuf,
    pub launch_target: LaunchTarget,
    pub artwork: Option<PathBuf>,
    #[serde(default)] pub metadata: BTreeMap<String, Value>,
    #[serde(default = "new_status")] pub sync_status: String,
}
fn new_status() -> String { "new".into() }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderDetection {
    pub installed: bool,
    pub launcher_path: Option<PathBuf>,
    pub data_paths: Vec<PathBuf>,
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub enabled: bool,
    pub detection: ProviderDetection,
    pub game_count: usize,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Default)]
pub struct ProviderScan { pub games: Vec<Game>, pub warnings: Vec<String>, pub complete: bool }

#[derive(Debug, Clone)]
pub enum ArtworkCandidate { Local(PathBuf), Remote(String) }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSnapshot {
    pub providers: Vec<ProviderInfo>,
    pub games: Vec<Game>,
    /// Only complete, enabled scans may establish that a game was uninstalled.
    pub authoritative_providers: Vec<String>,
    pub watch_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncPreview {
    pub added: Vec<Change>, pub updated: Vec<Change>, pub removed: Vec<Change>,
    pub unchanged: usize, pub warnings: Vec<String>, pub revision: String,
}
impl SyncPreview { pub fn changed(&self) -> bool { !(self.added.is_empty() && self.updated.is_empty() && self.removed.is_empty()) } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change { pub key: String, pub name: String, pub provider_id: String }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncResult {
    pub preview: SyncPreview, pub changed: bool, pub completed_at: u64,
    pub reload_error: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SunshineStatus {
    pub detected: bool, pub install_path: Option<PathBuf>, pub apps_path: Option<PathBuf>,
    pub version: Option<String>, pub service_name: Option<String>, pub service_status: String,
    pub applications: usize, pub managed: usize, pub error: Option<String>,
}
pub fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }
