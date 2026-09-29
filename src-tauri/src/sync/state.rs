use crate::{
    models::{GameKey, SyncResult},
    storage,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedEntry {
    pub key: GameKey,
    #[serde(default)]
    pub source_name: String,
    /// Local identity, not an invented Sunshine schema field.
    pub sunshine_entry_id: String,
    pub fields: Value,
    pub last_seen: u64,
    pub hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ManagedState {
    pub version: u32,
    pub apps_path: Option<PathBuf>,
    pub entries: BTreeMap<String, ManagedEntry>,
    pub last_sync: Option<SyncResult>,
}
impl Default for ManagedState {
    fn default() -> Self {
        Self {
            version: 1,
            apps_path: None,
            entries: BTreeMap::new(),
            last_sync: None,
        }
    }
}
impl ManagedState {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("state.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let state: Self = serde_json::from_slice(&fs::read(path)?).context(
            "state.json is invalid. Restore its backup before syncing; no ownership was assumed.",
        )?;
        if state.version != 1 {
            bail!("Unsupported state.json version");
        }
        for (key, entry) in &state.entries {
            let valid_fields = ["name", "cmd", "image-path", "working-dir"]
                .iter()
                .all(|field| entry.fields[*field].is_string())
                && entry.fields["detached"]
                    .as_array()
                    .is_some_and(|values| values.len() == 1 && values[0].is_string());
            if key != &entry.key.encoded()
                || entry.key.provider_id.is_empty()
                || entry.key.provider_game_id.is_empty()
                || !valid_fields
                || uuid::Uuid::parse_str(&entry.sunshine_entry_id).is_err()
                || entry.hash != storage::hash(&serde_json::to_vec(&entry.fields)?)
            {
                bail!("state.json contains an invalid ownership record; no changes were made");
            }
        }
        Ok(state)
    }
    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join("state.json");
        if path.exists() {
            storage::atomic_write(&root.join("state.json.bak"), &fs::read(path)?)?;
        }
        storage::write_json(&root.join("state.json"), self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub apps_path: PathBuf,
    pub before: String,
    pub after: String,
    pub state: ManagedState,
    #[serde(default)]
    pub backup_path: Option<PathBuf>,
    #[serde(default)]
    pub backup_skipped: bool,
}
/// Recover the state only if the exact intended apps transaction landed. If
/// someone edited Sunshine in the meantime, fail closed instead of guessing.
pub fn recover(root: &Path) -> Result<()> {
    let path = root.join("pending-sync.json");
    if !path.exists() {
        return Ok(());
    }
    let journal: Journal =
        serde_json::from_slice(&fs::read(&path)?).context("Sync recovery journal is invalid")?;
    let actual = storage::hash(&fs::read(&journal.apps_path)?);
    if actual == journal.after {
        journal.state.save(root)?;
    } else if actual != journal.before {
        let backup_hint = if journal.backup_skipped {
            "No backup was saved for this sync.".to_string()
        } else if let Some(backup) = &journal.backup_path {
            format!(
                "Restore the backup at {} before retrying.",
                backup.display()
            )
        } else {
            "Restore the earlier backup beside apps.json, if present, before retrying.".to_string()
        };
        bail!("Interrupted sync and external apps.json changes detected. {backup_hint} All files have been preserved.");
    }
    fs::remove_file(path)?;
    Ok(())
}
