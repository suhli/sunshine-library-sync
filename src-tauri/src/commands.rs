use crate::{
    app_state::{self, AppState, LibrarySnapshot},
    config::Settings,
    locale,
    models::*,
    network::{ConnectionTest, NetworkService},
    providers, sunshine,
};
use serde::Serialize;
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
type Cmd<T> = Result<T, String>;

#[derive(Serialize)]
pub struct RuntimeInfo {
    pub system_locale: String,
    pub fixture_mode: bool,
}

#[tauri::command]
pub fn get_runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        system_locale: locale::system_locale().into(),
        fixture_mode: cfg!(debug_assertions)
            && std::env::var_os("SUNSHINE_LIBRARY_SYNC_DATA").is_some(),
    }
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, Arc<AppState>>) -> LibrarySnapshot {
    state.snapshot.read().unwrap().clone()
}
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    state.settings()
}
#[tauri::command]
pub async fn scan_all_games(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Cmd<LibrarySnapshot> {
    let result = app_state::refresh(&app, &state)
        .await
        .map_err(|e| e.to_string())?;
    app_state::queue_artwork(app, state.inner().clone());
    Ok(result)
}
#[tauri::command]
pub async fn scan_provider(
    provider_id: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Cmd<LibrarySnapshot> {
    if !providers::registry(&state.settings())
        .iter()
        .any(|p| p.id() == provider_id)
    {
        return Err("Unknown provider".into());
    }
    // A single serialized registry refresh preserves a coherent cross-provider
    // view for collision handling. Each provider still reports failures locally.
    scan_all_games(app, state).await
}
#[tauri::command]
pub async fn get_sync_preview(app: AppHandle, state: State<'_, Arc<AppState>>) -> Cmd<SyncPreview> {
    let snapshot = app_state::refresh(&app, &state)
        .await
        .map_err(|e| e.to_string())?;
    snapshot.preview.ok_or_else(|| {
        snapshot
            .error
            .or(snapshot.sunshine.error)
            .unwrap_or_else(|| "Locate Sunshine apps.json before syncing".into())
    })
}
#[tauri::command]
pub async fn sync_sunshine(
    revision: Option<String>,
    game_key: Option<String>,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Cmd<SyncResult> {
    app_state::synchronize(&app, &state, revision, game_key)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Cmd<Settings> {
    {
        let _guard = state.operation.lock().await;
        state.check().map_err(|e| e.to_string())?;
        settings.validate().map_err(|e| e.to_string())?;
        let previous = state.settings();
        // Validate and persist before changing the autostart registration; roll
        // back the file if Windows rejects that registration.
        settings.save(&state.root).map_err(|e| e.to_string())?;
        if settings.general.start_with_windows != previous.general.start_with_windows {
            let result = if settings.general.start_with_windows {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            };
            if let Err(error) = result {
                let _ = previous.save(&state.root);
                return Err(format!("Unable to update Windows startup: {error}"));
            }
        }
        *state.settings.write().unwrap() = settings.clone();
    }
    crate::desktop::update_tray_locale(&app);
    app_state::refresh(&app, &state)
        .await
        .map_err(|e| e.to_string())?;
    app_state::queue_artwork(app, state.inner().clone());
    Ok(settings)
}
#[tauri::command]
pub async fn set_game_excluded(
    game_key: String,
    excluded: bool,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Cmd<Settings> {
    let mut settings = state.settings();
    if excluded {
        settings.excluded_games.insert(game_key);
    } else {
        settings.excluded_games.remove(&game_key);
    }
    save_settings(settings, app, state).await
}
#[tauri::command]
pub async fn restart_sunshine(state: State<'_, Arc<AppState>>) -> Cmd<()> {
    let _guard = state.operation.lock().await;
    tauri::async_runtime::spawn_blocking(sunshine::service::restart)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn test_proxy(network: crate::config::NetworkSettings) -> Cmd<ConnectionTest> {
    Ok(NetworkService::new(&network)
        .map_err(|e| e.to_string())?
        .test()
        .await)
}
#[tauri::command]
pub async fn get_artwork(game_key: String, state: State<'_, Arc<AppState>>) -> Cmd<Vec<u8>> {
    let game = state
        .snapshot
        .read()
        .unwrap()
        .scan
        .games
        .iter()
        .find(|g| g.key.encoded() == game_key)
        .cloned()
        .ok_or("Game not found")?;
    let path = crate::artwork::cache_path(&state.root, &game);
    tokio::fs::read(path)
        .await
        .map_err(|_| "No artwork cached".into())
}
#[tauri::command]
pub fn open_path(kind: String, key: Option<String>, state: State<'_, Arc<AppState>>) -> Cmd<()> {
    let target: PathBuf = match kind.as_str() {
        "logs" => {
            let p = state.root.join("logs");
            std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
            p
        }
        "data" => state.root.clone(),
        "sunshine" => sunshine::apps_path(&state.settings().sunshine)
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Invalid config path")?
            .into(),
        "game" => state
            .snapshot
            .read()
            .unwrap()
            .scan
            .games
            .iter()
            .find(|g| Some(g.key.encoded()) == key)
            .map(|g| g.install_path.clone())
            .ok_or("Game not found")?,
        "provider" => state
            .snapshot
            .read()
            .unwrap()
            .scan
            .providers
            .iter()
            .find(|p| Some(&p.id) == key.as_ref())
            .and_then(|p| {
                p.detection
                    .launcher_path
                    .clone()
                    .or_else(|| p.detection.data_paths.first().cloned())
            })
            .ok_or("Provider not found")?,
        _ => return Err("Unknown location".into()),
    };
    if !target.is_dir() {
        return Err("This folder is not available".into());
    }
    sunshine::service::hidden_command("explorer.exe")
        .arg(target)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
