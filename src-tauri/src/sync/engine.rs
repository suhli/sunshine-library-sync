use super::state::{self, Journal, ManagedEntry, ManagedState};
use crate::{config::Settings, models::*, storage, sunshine};
use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, fs::{self, OpenOptions}, path::{Path, PathBuf}};

pub struct Plan { pub document: Value, pub state: ManagedState, pub preview: SyncPreview }
pub fn state_directory(root: &Path, apps: &Path) -> PathBuf {
    let canonical = fs::canonicalize(apps).unwrap_or_else(|_| apps.to_path_buf());
    root.join("targets").join(storage::hash(canonical.to_string_lossy().to_lowercase().as_bytes()))
}
fn fields(game: &Game, name: &str) -> Value {
    json!({"name": name, "cmd": "", "detached": [game.launch_target.uri], "working-dir": game.install_path,
        "image-path": game.artwork.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()})
}
fn change(key: &str, entry: &ManagedEntry) -> Change {
    Change { key: key.into(), name: entry.fields["name"].as_str().unwrap_or(key).into(), provider_id: entry.key.provider_id.clone() }
}
fn matches_owned(app: &Value, owned: &Value) -> bool {
    owned.as_object().is_some_and(|fields| fields.iter().all(|(k, v)| app.get(k) == Some(v)))
}
pub fn build_plan(document: &Value, previous: &ManagedState, games: &[Game], authoritative: &[String], settings: &Settings) -> Result<Plan> {
    let mut merged = document.clone();
    let apps = merged.get_mut("apps").and_then(Value::as_array_mut).context("Sunshine apps.json must contain an apps array")?;
    let mut next = previous.clone();
    let mut preview = SyncPreview::default();
    let mut removal_indices = BTreeSet::new();
    let current: BTreeMap<_, _> = games.iter().map(|g| (g.key.encoded(), g)).collect();
    if current.len() != games.len() { bail!("Duplicate game IDs in scan; sync cancelled"); }
    let mut names = BTreeMap::<String, usize>::new();
    for game in games { *names.entry(game.name.to_lowercase()).or_default() += 1; }
    for (key, old) in &previous.entries {
        // Disabling or failing a provider freezes its entries; explicit game
        // exclusions still remove entries we can prove we own.
        let excluded = settings.excluded_games.contains(key);
        let can_remove = authoritative.contains(&old.key.provider_id) && settings.provider(&old.key.provider_id).enabled;
        if !excluded && !settings.provider(&old.key.provider_id).enabled { continue; }
        let indices: Vec<_> = apps.iter().enumerate().filter_map(|(i, a)| matches_owned(a, &old.fields).then_some(i)).collect();
        if indices.len() != 1 {
            preview.warnings.push(format!("{}: managed entry was edited, removed, or duplicated in Sunshine; left untouched", old.fields["name"].as_str().unwrap_or(key)));
            continue;
        }
        let index = indices[0];
        match current.get(key).filter(|_| !excluded) {
            None if excluded || can_remove => { removal_indices.insert(index); next.entries.remove(key); preview.removed.push(change(key, old)); },
            Some(game) => {
                let name = if names[&game.name.to_lowercase()] > 1 { format!("{} ({})", game.name, game.key.provider_id) } else { game.name.clone() };
                let desired = fields(game, &name);
                if desired != old.fields {
                    for (k, v) in desired.as_object().unwrap() { apps[index][k] = v.clone(); }
                    let mut updated = old.clone(); updated.fields = desired; updated.hash = storage::hash(&serde_json::to_vec(&updated.fields)?); updated.last_seen = now();
                    preview.updated.push(change(key, &updated)); next.entries.insert(key.clone(), updated);
                } else { preview.unchanged += 1; }
            }, _ => {},
        }
    }
    let mut index = 0; apps.retain(|_| { let keep = !removal_indices.contains(&index); index += 1; keep });
    for (key, game) in current {
        if previous.entries.contains_key(&key) || settings.excluded_games.contains(&key) || !settings.provider(&game.key.provider_id).enabled { continue; }
        if apps.iter().any(|a| a["detached"].as_array().is_some_and(|xs| xs.iter().any(|v| v.as_str() == Some(&game.launch_target.uri))) || a["cmd"].as_str() == Some(&game.launch_target.uri)) {
            preview.warnings.push(format!("{}: an existing Sunshine entry uses this launch target; it remains unmanaged", game.name)); continue;
        }
        let mut name = game.name.clone();
        if names[&name.to_lowercase()] > 1 || apps.iter().any(|a| a["name"].as_str() == Some(&name)) { name = format!("{} ({})", game.name, game.key.provider_id); }
        if apps.iter().any(|a| a["name"].as_str() == Some(&name)) { name = format!("{} [{}]", name, game.key.provider_game_id); }
        let desired = fields(game, &name);
        let entry = ManagedEntry { key: game.key.clone(), sunshine_entry_id: uuid::Uuid::new_v4().to_string(), hash: storage::hash(&serde_json::to_vec(&desired)?), fields: desired.clone(), last_seen: now() };
        apps.push(desired); preview.added.push(change(&key, &entry)); next.entries.insert(key, entry);
    }
    Ok(Plan { document: merged, state: next, preview })
}

fn lock(root: &Path) -> Result<std::fs::File> {
    fs::create_dir_all(root)?;
    let file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("sync.lock"))?;
    file.try_lock_exclusive().context("Another sync is in progress")?; Ok(file)
}
pub fn sync(root: &Path, settings: &Settings, snapshot: &ScanSnapshot, apply: bool, expected_revision: Option<&str>) -> Result<SyncResult> {
    let path = sunshine::apps_path(&settings.sunshine)?;
    let state_root = state_directory(root, &path);
    let _guard = lock(&state_root)?;
    state::recover(&state_root)?;
    let previous = ManagedState::load(&state_root)?;
    let (original, document) = sunshine::read_apps(&path)?;
    let mut plan = build_plan(&document, &previous, &snapshot.games, &snapshot.authoritative_providers, settings)?;
    let revision_bytes = [original.as_slice(), serde_json::to_vec(&plan.document)?.as_slice()].concat();
    plan.preview.revision = storage::hash(&revision_bytes);
    for provider in &snapshot.providers {
        if let Some(error) = &provider.error { plan.preview.warnings.push(format!("{}: {error}; existing entries preserved", provider.display_name)); }
        if !provider.warnings.is_empty() { plan.preview.warnings.push(format!("{}: scan incomplete; removals paused", provider.display_name)); }
    }
    if let Some(expected) = expected_revision { if plan.preview.revision != expected { bail!("Library or Sunshine changed since this preview. Refresh the preview before applying."); } }
    let changed = document != plan.document;
    let mut result = SyncResult { preview: plan.preview, changed, completed_at: now(), reload_error: None };
    if !apply { return Ok(result); }
    if changed {
        let mut bytes = serde_json::to_vec_pretty(&plan.document)?; bytes.push(b'\n');
        plan.state.apps_path = Some(path.clone()); plan.state.last_sync = Some(result.clone());
        let journal = Journal { apps_path: path.clone(), before: storage::hash(&original), after: storage::hash(&bytes), state: plan.state.clone() };
        storage::write_json(&state_root.join("pending-sync.json"), &journal)?;
        // An external editor may not honor our lock. Detect changes immediately
        // before replacement. No destructive delete/rename fallback is allowed.
        if fs::read(&path)? != original { bail!("Sunshine apps.json changed during sync. Retry after closing its editor."); }
        let backup = path.with_file_name(format!("{}.bak", path.file_name().context("Invalid apps path")?.to_string_lossy()));
        storage::atomic_write(&backup, &original).context("Unable to back up Sunshine apps.json")?;
        if fs::read(&path)? != original { bail!("Sunshine apps.json changed during backup. Sync cancelled."); }
        storage::atomic_write(&path, &bytes).context("Sunshine apps.json was not updated; check write permissions")?;
        plan.state.save(&state_root).context("apps.json was written; state recovery will run on next sync")?;
        fs::remove_file(state_root.join("pending-sync.json"))?;
        for change in &result.preview.added { tracing::info!("[Sync] Added {}", change.name); }
        for change in &result.preview.updated { tracing::info!("[Sync] Updated {}", change.name); }
        for change in &result.preview.removed { tracing::info!("[Sync] Removed {}", change.name); }
        if let Err(e) = sunshine::service::reload(&settings.sunshine) { result.reload_error = Some(e.to_string()); }
    }
    // Only state (last sync time) changes on a no-op. apps.json and its backup
    // remain byte-identical, and no reload is issued.
    plan.state.last_sync = Some(result.clone()); plan.state.save(&state_root)?;
    Ok(result)
}
pub fn annotate(games: &mut [Game], state: &ManagedState, settings: &Settings, preview: Option<&SyncPreview>) {
    for game in games {
        let key = game.key.encoded();
        game.sync_status = if settings.excluded_games.contains(&key) { "excluded" }
        else if preview.is_some_and(|p| p.updated.iter().any(|c| c.key == key)) { "changed" }
        else if state.entries.contains_key(&key) { "synced" } else { "new" }.into();
    }
}
