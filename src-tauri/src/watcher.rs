use crate::app_state::{self, AppState};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{collections::BTreeSet, path::PathBuf, sync::{Arc, atomic::Ordering}, time::Duration};
use tauri::{AppHandle, Emitter};

fn relevant(event: &Event) -> bool {
    !matches!(event.kind, EventKind::Access(_)) && event.paths.iter().any(|p| {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        name == "libraryfolders.vdf" || name.ends_with(".acf") || name.ends_with(".item") || p.extension().is_none()
    })
}
pub fn start(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let mut paths_rx = state.watch_paths.subscribe();
        let mut watcher = match RecommendedWatcher::new(move |result: notify::Result<Event>| {
            if result.as_ref().map(relevant).unwrap_or(true) { let _ = tx.try_send(result.map(|_| ()).map_err(|e| e.to_string())); }
        }, notify::Config::default()) { Ok(w) => w, Err(e) => { let _ = app.emit("background-error", format!("File watcher failed: {e}")); return; } };
        let mut watched = BTreeSet::<PathBuf>::new();
        let mut pending: Option<tokio::time::Instant> = None;
        loop {
            let desired: BTreeSet<_> = paths_rx.borrow_and_update().iter().filter_map(|p| {
                let mut path = p.clone(); while !path.is_dir() { if !path.pop() { return None; } } Some(path)
            }).collect();
            for old in watched.difference(&desired) { let _ = watcher.unwatch(old); }
            let mut successful = BTreeSet::new();
            for new in &desired {
                if watched.contains(new) || watcher.watch(new, RecursiveMode::NonRecursive).is_ok() { successful.insert(new.clone()); }
                else { let _ = app.emit("background-error", format!("Unable to watch {}", new.display())); }
            }
            watched = successful;
            let deadline = pending.unwrap_or_else(|| tokio::time::Instant::now() + Duration::from_secs(86400));
            tokio::select! {
                _ = state.cancel.cancelled() => break,
                result = paths_rx.changed() => { if result.is_err() { break; } },
                event = rx.recv() => {
                    match event { Some(Ok(())) => {}, Some(Err(e)) => { let _ = app.emit("background-error", format!("Watcher: {e}")); }, None => break }
                    pending = Some(tokio::time::Instant::now() + Duration::from_secs(3));
                },
                _ = tokio::time::sleep_until(deadline), if pending.is_some() => {
                    pending = None;
                    let result = if state.settings().general.auto_sync && !state.paused.load(Ordering::Relaxed) {
                        app_state::synchronize(&app, &state, None, None).await.map(|_| ())
                    } else { app_state::refresh(&app, &state).await.map(|_| ()) };
                    if let Err(e) = result { let _ = app.emit("background-error", e.to_string()); }
                    app_state::queue_artwork(app.clone(), state.clone());
                }
            }
        }
        // Dropping the watcher closes its native notification handles.
        drop(watcher);
    });
}
