use crate::{artwork, config::Settings, models::*, provider_registry, sync::{engine, state::ManagedState}, sunshine};
use anyhow::{bail, Result};
use serde::Serialize;
use std::{path::{Path, PathBuf}, sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, watch};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibrarySnapshot { pub scan: ScanSnapshot, pub sunshine: SunshineStatus, pub last_sync: Option<SyncResult>, pub preview: Option<SyncPreview>, pub error: Option<String>, pub scanning: bool }
pub struct AppState {
    pub root: PathBuf, pub settings: RwLock<Settings>, pub snapshot: RwLock<LibrarySnapshot>,
    pub operation: Mutex<()>, pub cancel: CancellationToken, pub paused: AtomicBool,
    pub artwork_busy: AtomicBool, pub watch_paths: watch::Sender<Vec<PathBuf>>, pub startup_error: Option<String>,
}
impl AppState {
    pub fn new(root: PathBuf) -> Arc<Self> {
        let loaded = Settings::load(&root);
        let startup_error = loaded.as_ref().err().map(|e| e.to_string());
        let (sender, _) = watch::channel(vec![]);
        Arc::new(Self { root, settings: RwLock::new(loaded.unwrap_or_default()), snapshot: RwLock::new(LibrarySnapshot { scanning: true, error: startup_error.clone(), ..Default::default() }), operation: Mutex::new(()), cancel: CancellationToken::new(), paused: AtomicBool::new(false), artwork_busy: AtomicBool::new(false), watch_paths: sender, startup_error })
    }
    pub fn check(&self) -> Result<()> { if let Some(error) = &self.startup_error { bail!("{error}"); } if self.cancel.is_cancelled() { bail!("Application is exiting"); } Ok(()) }
    pub fn settings(&self) -> Settings { self.settings.read().unwrap().clone() }
    pub fn emit(&self, app: &AppHandle) { let _ = app.emit("library-updated", self.snapshot.read().unwrap().clone()); }
}
fn collect(root: &Path, settings: &Settings) -> LibrarySnapshot {
    let mut scan = provider_registry::scan(settings);
    for game in &mut scan.games { game.artwork = artwork::cached(root, game); }
    let managed = sunshine::apps_path(&settings.sunshine).map(|p| engine::state_directory(root, &p)).and_then(|p| ManagedState::load(&p));
    let mut error = managed.as_ref().err().map(|e| e.to_string());
    let managed = managed.unwrap_or_default();
    let status = sunshine::status(&settings.sunshine, managed.entries.len());
    let preview = if status.detected && error.is_none() {
        match engine::sync(root, settings, &scan, false, None) { Ok(r) => Some(r.preview), Err(e) => { error = Some(e.to_string()); None } }
    } else { None };
    engine::annotate(&mut scan.games, &managed, settings, preview.as_ref());
    LibrarySnapshot { scan, sunshine: status, last_sync: managed.last_sync, preview, error, scanning: false }
}
pub async fn refresh_inner(app: &AppHandle, state: &Arc<AppState>) -> Result<LibrarySnapshot> {
    state.check()?;
    let root = state.root.clone(); let settings = state.settings();
    let snapshot = tauri::async_runtime::spawn_blocking(move || collect(&root, &settings)).await?;
    state.watch_paths.send_replace(snapshot.scan.watch_paths.clone());
    *state.snapshot.write().unwrap() = snapshot.clone(); state.emit(app); Ok(snapshot)
}
pub async fn refresh(app: &AppHandle, state: &Arc<AppState>) -> Result<LibrarySnapshot> {
    let _guard = state.operation.lock().await;
    refresh_inner(app, state).await
}
pub async fn synchronize(app: &AppHandle, state: &Arc<AppState>, expected: Option<String>, only_key: Option<String>) -> Result<SyncResult> {
    let _guard = state.operation.lock().await; state.check()?;
    let snapshot = refresh_inner(app, state).await?;
    if let Some(error) = snapshot.error { bail!("{error}"); }
    let mut scan = snapshot.scan; let mut settings = state.settings(); let root = state.root.clone();
    if let Some(key) = only_key { scan.games.retain(|g| g.key.encoded() == key); scan.authoritative_providers.clear(); settings.excluded_games.retain(|k| k == &key); }
    let result = tauri::async_runtime::spawn_blocking(move || engine::sync(&root, &settings, &scan, true, expected.as_deref())).await??;
    refresh_inner(app, state).await?;
    let _ = app.emit("sync-completed", &result); Ok(result)
}
pub fn queue_artwork(app: AppHandle, state: Arc<AppState>) {
    if state.artwork_busy.swap(true, Ordering::SeqCst) { return; }
    tauri::async_runtime::spawn(async move {
        let settings = state.settings();
        let games = state.snapshot.read().unwrap().scan.games.clone(); let mut changed = false;
        for game in games {
            if state.cancel.is_cancelled() { break; }
            if game.artwork.is_some() { continue; }
            let result = tokio::select! { _ = state.cancel.cancelled() => break, result = artwork::fetch(&state.root, &settings, &game) => result };
            match result {
                Ok(Some(_)) => { changed = true; let _ = app.emit("artwork-updated", game.key.encoded()); },
                Ok(None) => tracing::debug!("[Artwork] No cover for {}", game.key.encoded()),
                Err(_) => tracing::warn!("[Artwork] Download failed for {}; local sync remains available", game.key.encoded()),
            }
        }
        if changed && !state.cancel.is_cancelled() {
            let _ = refresh(&app, &state).await;
            if state.settings().general.auto_sync && !state.paused.load(Ordering::Relaxed) {
                if let Err(e) = synchronize(&app, &state, None, None).await { let _ = app.emit("background-error", e.to_string()); }
            }
        }
        state.artwork_busy.store(false, Ordering::SeqCst);
    });
}
