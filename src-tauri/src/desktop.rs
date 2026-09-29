use crate::{
    app_state::{self, AppState},
    commands, config, locale,
    sync::engine::BackupFailure,
    watcher,
};

struct TrayItems {
    open: MenuItem<tauri::Wry>,
    sync: MenuItem<tauri::Wry>,
    pause: CheckMenuItem<tauri::Wry>,
    status: MenuItem<tauri::Wry>,
    exit: MenuItem<tauri::Wry>,
}

pub(crate) fn update_tray_locale(app: &tauri::AppHandle) {
    let items = app.state::<TrayItems>();
    let state = app.state::<Arc<AppState>>();
    let language = locale::selected_locale(&state.settings()).to_owned();
    let service = state
        .snapshot
        .read()
        .unwrap()
        .sunshine
        .service_status
        .clone();
    let _ = items.open.set_text(locale::tray_label(&language, "open"));
    let _ = items.sync.set_text(locale::tray_label(&language, "sync"));
    let _ = items.pause.set_text(locale::tray_label(&language, "pause"));
    let _ = items
        .status
        .set_text(locale::tray_status(&language, &service));
    let _ = items.exit.set_text(locale::tray_label(&language, "exit"));
}
use std::sync::{atomic::Ordering, Arc};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

fn show(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
fn quit(app: tauri::AppHandle) {
    let state = app.state::<Arc<AppState>>().inner().clone();
    state.cancel.cancel();
    tauri::async_runtime::spawn(async move {
        // Let an in-flight atomic sync finish before terminating the process.
        let _guard = state.operation.lock().await;
        app.exit(0);
    });
}
pub fn run() {
    let root = config::data_root();
    let _ = std::fs::create_dir_all(root.join("logs"));
    let log = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("library-sync")
        .filename_suffix("log")
        .max_log_files(7)
        .build(root.join("logs"));
    let _log_guard = log.ok().map(|log| {
        let (writer, guard) = tracing_appender::non_blocking(log);
        let _ = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(writer)
            .try_init();
        guard
    });
    let state = AppState::new(root);
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_runtime_info,
            commands::get_snapshot,
            commands::get_settings,
            commands::scan_all_games,
            commands::scan_provider,
            commands::get_sync_preview,
            commands::sync_sunshine,
            commands::save_settings,
            commands::set_game_excluded,
            commands::restart_sunshine,
            commands::test_proxy,
            commands::get_artwork,
            commands::open_path
        ])
        .setup(|app| {
            let state = app.state::<Arc<AppState>>().inner().clone();
            let language = locale::selected_locale(&state.settings()).to_owned();
            let open = MenuItem::with_id(
                app,
                "open",
                locale::tray_label(&language, "open"),
                true,
                None::<&str>,
            )?;
            let sync = MenuItem::with_id(
                app,
                "sync",
                locale::tray_label(&language, "sync"),
                true,
                None::<&str>,
            )?;
            let pause = CheckMenuItem::with_id(
                app,
                "pause",
                locale::tray_label(&language, "pause"),
                true,
                false,
                None::<&str>,
            )?;
            let status = MenuItem::with_id(
                app,
                "status",
                locale::tray_status(&language, "detecting"),
                false,
                None::<&str>,
            )?;
            let exit = MenuItem::with_id(
                app,
                "exit",
                locale::tray_label(&language, "exit"),
                true,
                None::<&str>,
            )?;
            let menu = Menu::with_items(app, &[&open, &sync, &pause, &status, &exit])?;
            app.manage(TrayItems {
                open,
                sync,
                pause,
                status: status.clone(),
                exit,
            });
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Sunshine Library Sync")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show(app),
                    "exit" => quit(app.clone()),
                    "pause" => {
                        let state = app.state::<Arc<AppState>>();
                        let paused = !state.paused.load(Ordering::Relaxed);
                        state.paused.store(paused, Ordering::Relaxed);
                        let _ = app.emit("auto-sync-paused", paused);
                    }
                    "sync" => {
                        let app = app.clone();
                        let state = app.state::<Arc<AppState>>().inner().clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) =
                                app_state::synchronize(&app, &state, None, None, false).await
                            {
                                if let Some(failure) = e.downcast_ref::<BackupFailure>() {
                                    show(&app);
                                    let _ = app.emit(
                                        "sync-backup-failed",
                                        serde_json::json!({
                                            "backup_path": failure.path.to_string_lossy(),
                                            "reason": failure.reason,
                                            "revision": failure.revision,
                                            "game_key": null,
                                        }),
                                    );
                                } else {
                                    let _ = app.emit("background-error", format!("{e:#}"));
                                }
                            }
                        });
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                        show(tray.app_handle());
                    }
                })
                .build(app)?;
            use tauri::Listener;
            let status_handle = app.handle().clone();
            app.listen("library-updated", move |event| {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let service = value["sunshine"]["service_status"]
                        .as_str()
                        .unwrap_or("unknown");
                    let state = status_handle.state::<Arc<AppState>>();
                    let language = locale::selected_locale(&state.settings()).to_owned();
                    let _ = status.set_text(locale::tray_status(&language, service));
                }
            });
            let handle = app.handle().clone();
            watcher::start(handle.clone(), state.clone());
            if state.settings().general.start_minimized
                && std::env::args().any(|a| a == "--autostart")
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }
            tauri::async_runtime::spawn(async move {
                if let Err(e) = app_state::refresh(&handle, &state).await {
                    let mut snapshot = state.snapshot.write().unwrap();
                    snapshot.scanning = false;
                    snapshot.error = Some(e.to_string());
                    drop(snapshot);
                    state.emit(&handle);
                }
                if state.startup_error.is_none() && state.settings().general.auto_sync {
                    if let Err(e) = app_state::synchronize(&handle, &state, None, None, false).await
                    {
                        let _ = handle.emit("background-error", e.to_string());
                    }
                }
                app_state::queue_artwork(handle, state);
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle();
                let state = app.state::<Arc<AppState>>();
                if state.settings().general.close_behavior == "tray" {
                    let _ = window.hide();
                } else {
                    quit(app.clone());
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Unable to initialize Sunshine Library Sync")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<Arc<AppState>>().cancel.cancel();
            }
        });
}
