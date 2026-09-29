use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use sunshine_library_sync::{
    config::{ProviderSettings, Settings},
    models::*,
    network::NetworkService,
    provider_registry,
    providers::{self, epic, steam, vdf, GameProvider},
    storage, sunshine,
    sync::{
        engine,
        state::{self, Journal, ManagedState},
    },
};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/apps.json")).unwrap()
}

#[test]
fn failed_backup_never_advances_state_or_changes_apps() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    storage::write_json(&path, &fixture()).unwrap();
    let before = fs::read(&path).unwrap();
    fs::create_dir(temp.path().join("apps.json.bak")).unwrap();
    let mut settings = Settings::default();
    settings.sunshine.apps_path = Some(path.clone());
    let snapshot = ScanSnapshot {
        games: vec![game("steam", "1", "Game")],
        ..Default::default()
    };
    let error = engine::sync(temp.path(), &settings, &snapshot, true, None).unwrap_err();
    let failure = error.downcast_ref::<engine::BackupFailure>().unwrap();
    assert_eq!(failure.path, temp.path().join("apps.json.bak"));
    assert!(!failure.reason.is_empty());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(!engine::state_directory(temp.path(), &path)
        .join("pending-sync.json")
        .exists());
    assert!(
        ManagedState::load(&engine::state_directory(temp.path(), &path))
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn confirmed_backup_failure_rechecks_revision_before_unbacked_sync() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    storage::write_json(&path, &fixture()).unwrap();
    fs::create_dir(temp.path().join("apps.json.bak")).unwrap();
    let mut settings = Settings::default();
    settings.sunshine.apps_path = Some(path.clone());
    let snapshot = ScanSnapshot {
        games: vec![game("steam", "1", "Game")],
        ..Default::default()
    };
    let revision = engine::sync(temp.path(), &settings, &snapshot, false, None)
        .unwrap()
        .preview
        .revision;
    assert!(
        engine::sync_with_backup_policy(temp.path(), &settings, &snapshot, true, None, true)
            .is_err()
    );
    let mut external = fixture();
    external["apps"][0]["name"] = "Externally edited".into();
    storage::write_json(&path, &external).unwrap();
    assert!(engine::sync_with_backup_policy(
        temp.path(),
        &settings,
        &snapshot,
        true,
        Some(&revision),
        true
    )
    .is_err());
    assert_eq!(sunshine::read_apps(&path).unwrap().1, external);
    storage::write_json(&path, &fixture()).unwrap();
    let result = engine::sync_with_backup_policy(
        temp.path(),
        &settings,
        &snapshot,
        true,
        Some(&revision),
        true,
    )
    .unwrap();
    assert!(result.changed);
    assert_eq!(result.preview.added.len(), 1);
    assert!(temp.path().join("apps.json.bak").is_dir());
    assert_eq!(
        sunshine::read_apps(&path).unwrap().1["apps"]
            .as_array()
            .unwrap()
            .len(),
        fixture()["apps"].as_array().unwrap().len() + 1
    );
    assert_eq!(
        ManagedState::load(&engine::state_directory(temp.path(), &path))
            .unwrap()
            .entries
            .len(),
        1
    );
}

#[test]
fn invalid_ownership_record_cannot_match_a_manual_app() {
    let temp = tempfile::tempdir().unwrap();
    let p = plan(
        &fixture(),
        &ManagedState::default(),
        &[game("steam", "1", "Game")],
    );
    let mut state = p.state;
    state.entries.get_mut("steam:1").unwrap().fields = json!({});
    storage::write_json(&temp.path().join("state.json"), &state).unwrap();
    assert!(ManagedState::load(temp.path()).is_err());
}

#[tokio::test]
async fn failed_artwork_and_unreachable_proxy_do_not_block_local_sync() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("steamapps")).unwrap();
    let mut settings = Settings::default();
    settings.providers.insert(
        "steam".into(),
        ProviderSettings {
            enabled: true,
            path: Some(temp.path().into()),
        },
    );
    settings.network.proxy_mode = "custom".into();
    settings.network.proxy_url = "http://127.0.0.1:1".into();
    let g = game("steam", "9999999999999", "Offline Game");
    assert!(
        sunshine_library_sync::artwork::fetch(temp.path(), &settings, &g)
            .await
            .is_err()
    );
    let result =
        engine::build_plan(&fixture(), &ManagedState::default(), &[g], &[], &settings).unwrap();
    assert_eq!(result.preview.added.len(), 1);
}
fn game(provider: &str, id: &str, name: &str) -> Game {
    Game {
        key: GameKey {
            provider_id: provider.into(),
            provider_game_id: id.into(),
        },
        name: name.into(),
        install_path: PathBuf::from("C:/Games").join(id),
        manifest_path: PathBuf::from("manifest"),
        launch_target: LaunchTarget {
            uri: format!("{provider}://game/{id}"),
        },
        artwork: None,
        metadata: BTreeMap::new(),
        sync_status: "new".into(),
    }
}
fn plan(doc: &Value, state: &ManagedState, games: &[Game]) -> engine::Plan {
    engine::build_plan(
        doc,
        state,
        games,
        &["steam".into(), "epic".into()],
        &Settings::default(),
    )
    .unwrap()
}
fn epic_manifest(root: &Path) -> Value {
    let mut data: Value = serde_json::from_str(include_str!("fixtures/epic.item")).unwrap();
    data["InstallLocation"] = json!(root);
    data
}

#[test]
fn vdf_supports_comments_escapes_and_nested_objects() {
    let v = vdf::parse(
        r#"// comment
    "root" { "path" "D:\\Library" "name" "A \"quoted\" game" child { bare value } }"#,
    )
    .unwrap();
    let root = v["root"].object().unwrap();
    assert_eq!(vdf::text(root, "path").unwrap(), r"D:\Library");
    assert_eq!(vdf::text(root, "name").unwrap(), "A \"quoted\" game");
}
#[test]
fn vdf_rejects_incomplete_and_duplicate_metadata() {
    for text in ["\"x\" {", "\"name\" \"unterminated", "x y x z", "}"] {
        assert!(vdf::parse(text).is_err());
    }
}
#[test]
fn steam_scans_multiple_libraries_and_removal() {
    let temp = tempfile::tempdir().unwrap();
    let primary = temp.path().join("Steam");
    let secondary = temp.path().join("Other");
    for p in [&primary, &secondary] {
        fs::create_dir_all(p.join("steamapps/common/Cyberpunk 2077")).unwrap();
    }
    let folders = format!(
        "\"libraryfolders\" {{ \"0\" {{ \"path\" \"{}\" }} \"1\" {{ \"path\" \"{}\" }} }}",
        primary.display(),
        secondary.display()
    );
    fs::write(primary.join("steamapps/libraryfolders.vdf"), folders).unwrap();
    fs::write(
        primary.join("steamapps/appmanifest_1091500.acf"),
        include_str!("fixtures/steam.acf"),
    )
    .unwrap();
    let second = secondary.join("steamapps/appmanifest_42.acf");
    fs::write(
        &second,
        include_str!("fixtures/steam.acf").replace("1091500", "42"),
    )
    .unwrap();
    let provider = steam::SteamProvider::new(Some(primary));
    let detection = provider.detect().unwrap();
    assert_eq!(detection.data_paths.len(), 2);
    let scan = provider.scan_games(&detection).unwrap();
    assert!(scan.complete);
    assert_eq!(scan.games.len(), 2);
    fs::remove_file(second).unwrap();
    assert_eq!(provider.scan_games(&detection).unwrap().games.len(), 1);
}
#[test]
fn steam_non_game_apps_are_not_games_or_sync_candidates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("steamapps/common/Cyberpunk 2077")).unwrap();
    fs::create_dir_all(root.join("steamapps/common/Steamworks Common Redistributables")).unwrap();
    fs::create_dir_all(root.join("steamapps/common/Wallpaper Engine")).unwrap();
    fs::write(
        root.join("steamapps/appmanifest_1091500.acf"),
        include_str!("fixtures/steam.acf"),
    )
    .unwrap();
    fs::write(
        root.join("steamapps/appmanifest_228980.acf"),
        include_str!("fixtures/steam.acf")
            .replace("1091500", "228980")
            .replace("Cyberpunk 2077", "Steamworks Common Redistributables"),
    )
    .unwrap();
    fs::write(
        root.join("steamapps/appmanifest_431960.acf"),
        include_str!("fixtures/steam.acf")
            .replace("1091500", "431960")
            .replace("Cyberpunk 2077", "Wallpaper Engine"),
    )
    .unwrap();

    let registry: Vec<Box<dyn GameProvider>> =
        vec![Box::new(steam::SteamProvider::new(Some(root.into())))];
    let snapshot = provider_registry::scan_providers(&registry, &Settings::default());
    assert!(snapshot.authoritative_providers.contains(&"steam".into()));
    assert_eq!(snapshot.games.len(), 1);
    assert_eq!(snapshot.games[0].key.provider_game_id, "1091500");
    assert_eq!(snapshot.providers[0].game_count, 1);
    assert_eq!(
        plan(&fixture(), &ManagedState::default(), &snapshot.games)
            .preview
            .added
            .len(),
        1
    );

    for name in ["Steamworks Common Redistributables", "Wallpaper Engine"] {
        let renamed_id = include_str!("fixtures/steam.acf").replace("Cyberpunk 2077", name);
        assert!(
            steam::parse_manifest(&renamed_id, Path::new("app.acf"), root)
                .unwrap()
                .is_none()
        );
    }
    let similarly_named =
        include_str!("fixtures/steam.acf").replace("Cyberpunk 2077", "Wallpaper Engine 2");
    fs::create_dir_all(root.join("steamapps/common/Wallpaper Engine 2")).unwrap();
    assert!(
        steam::parse_manifest(&similarly_named, Path::new("app.acf"), root)
            .unwrap()
            .is_some()
    );
}
#[test]
fn provider_count_uses_deduplicated_games_across_steam_libraries() {
    let temp = tempfile::tempdir().unwrap();
    let primary = temp.path().join("Steam");
    let secondary = temp.path().join("Other");
    for root in [&primary, &secondary] {
        fs::create_dir_all(root.join("steamapps/common/Cyberpunk 2077")).unwrap();
        fs::write(
            root.join("steamapps/appmanifest_1091500.acf"),
            include_str!("fixtures/steam.acf"),
        )
        .unwrap();
    }
    fs::write(
        primary.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}",
            secondary.display()
        ),
    )
    .unwrap();

    let registry: Vec<Box<dyn GameProvider>> =
        vec![Box::new(steam::SteamProvider::new(Some(primary)))];
    let snapshot = provider_registry::scan_providers(&registry, &Settings::default());
    assert_eq!(snapshot.games.len(), 1);
    assert_eq!(snapshot.providers[0].game_count, 1);
}
#[test]
fn missing_steam_drive_prevents_authoritative_removal() {
    let temp = tempfile::tempdir().unwrap();
    let steam = temp.path().join("Steam");
    fs::create_dir_all(steam.join("steamapps")).unwrap();
    fs::write(
        steam.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}",
            temp.path().join("offline-drive").display()
        ),
    )
    .unwrap();
    let mut s = Settings::default();
    s.providers.insert(
        "steam".into(),
        ProviderSettings {
            enabled: true,
            path: Some(steam),
        },
    );
    s.providers.insert(
        "epic".into(),
        ProviderSettings {
            enabled: false,
            path: None,
        },
    );
    let result = provider_registry::scan(&s);
    assert!(!result
        .authoritative_providers
        .contains(&"steam".to_string()));
    assert!(!result.providers[0].warnings.is_empty());
}
#[test]
fn corrupt_manifest_keeps_other_games_but_prevents_removal() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("steamapps/common/Cyberpunk 2077")).unwrap();
    fs::write(
        temp.path().join("steamapps/appmanifest_1091500.acf"),
        include_str!("fixtures/steam.acf"),
    )
    .unwrap();
    fs::write(temp.path().join("steamapps/appmanifest_9.acf"), "bad {").unwrap();
    let p = steam::SteamProvider::new(Some(temp.path().into()));
    let s = p.scan_games(&p.detect().unwrap()).unwrap();
    assert_eq!(s.games.len(), 1);
    assert!(!s.complete);
}
#[test]
fn steam_missing_install_directory_is_not_installed() {
    let temp = tempfile::tempdir().unwrap();
    assert!(steam::parse_manifest(
        include_str!("fixtures/steam.acf"),
        Path::new("app.acf"),
        temp.path()
    )
    .unwrap()
    .is_none());
}
#[test]
fn steam_rejects_traversal_and_transitional_installs() {
    let temp = tempfile::tempdir().unwrap();
    for manifest in [
        include_str!("fixtures/steam.acf").replace("\"4\"", "\"2\""),
        include_str!("fixtures/steam.acf").replace(
            "\"installdir\" \"Cyberpunk 2077\"",
            "\"installdir\" \"../escape\"",
        ),
    ] {
        assert!(steam::parse_manifest(&manifest, Path::new("app.acf"), temp.path()).is_err());
    }
}
#[test]
fn epic_launch_uri_encodes_namespace_item_and_app() {
    let temp = tempfile::tempdir().unwrap();
    let parsed = epic::parse_manifest(
        &epic_manifest(temp.path()).to_string(),
        Path::new("game.item"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(parsed.launch_target.uri, "com.epicgames.launcher://apps/test-namespace%3Acatalog-id%3AA%20Game%3AWith%20Spaces?action=launch&silent=true");
}
#[test]
fn epic_filters_engine_components_dlc_and_addons() {
    let temp = tempfile::tempdir().unwrap();
    for category in ["engines", "plugins", "dlc", "addons"] {
        let mut data = epic_manifest(temp.path());
        data["AppCategories"] = json!([category]);
        data["bIsExecutable"] = false.into();
        assert!(
            epic::parse_manifest(&data.to_string(), Path::new("game.item"))
                .unwrap()
                .is_none(),
            "{category}"
        );
    }
}
#[test]
fn epic_allows_explicitly_launchable_addon() {
    let temp = tempfile::tempdir().unwrap();
    let mut data = epic_manifest(temp.path());
    data["AppCategories"] = json!(["dlc"]);
    assert!(
        epic::parse_manifest(&data.to_string(), Path::new("game.item"))
            .unwrap()
            .is_some()
    );
}
#[test]
fn epic_incomplete_install_is_not_an_authoritative_uninstall() {
    let temp = tempfile::tempdir().unwrap();
    let mut data = epic_manifest(temp.path());
    data["bIsIncompleteInstall"] = true.into();
    assert!(epic::parse_manifest(&data.to_string(), Path::new("game.item")).is_err());
}
#[test]
fn game_key_is_provider_scoped() {
    assert_ne!(
        game("steam", "42", "Same").key,
        game("epic", "42", "Same").key
    );
    assert_eq!(game("steam", "42", "A").key, game("steam", "42", "B").key);
}

#[test]
fn merge_preserves_manual_apps_and_unknown_root_fields() {
    let original = fixture();
    let p = plan(
        &original,
        &ManagedState::default(),
        &[game("steam", "1", "Game")],
    );
    assert_eq!(
        &p.document["apps"].as_array().unwrap()[..3],
        original["apps"].as_array().unwrap()
    );
    assert_eq!(p.document["env"], original["env"]);
    assert_eq!(p.document["custom-root"], original["custom-root"]);
}
#[test]
fn repeated_sync_is_idempotent_even_with_name_collisions() {
    for games in [
        vec![game("steam", "1", "Desktop")],
        vec![game("steam", "1", "Same"), game("epic", "1", "Same")],
        vec![game("steam", "1", "Same"), game("steam", "2", "Same")],
    ] {
        let first = plan(&fixture(), &ManagedState::default(), &games);
        let second = plan(&first.document, &first.state, &games);
        assert_eq!(first.document, second.document);
        assert!(!second.preview.changed());
    }
}
#[test]
fn uninstalled_game_only_deletes_owned_entry() {
    let p = plan(
        &fixture(),
        &ManagedState::default(),
        &[game("steam", "1", "Game")],
    );
    let removed = plan(&p.document, &p.state, &[]);
    assert_eq!(removed.document, fixture());
    assert_eq!(removed.preview.removed.len(), 1);
}
#[test]
fn failed_or_disabled_provider_retains_managed_entries() {
    let p = plan(
        &fixture(),
        &ManagedState::default(),
        &[game("steam", "1", "Game")],
    );
    let mut settings = Settings::default();
    for disabled in [false, true] {
        settings.providers.insert(
            "steam".into(),
            ProviderSettings {
                enabled: !disabled,
                path: None,
            },
        );
        let next = engine::build_plan(&p.document, &p.state, &[], &[], &settings).unwrap();
        assert_eq!(next.document, p.document);
    }
}
#[test]
fn exclusion_removes_only_the_managed_entry() {
    let g = game("steam", "1", "Desktop");
    let p = plan(
        &fixture(),
        &ManagedState::default(),
        std::slice::from_ref(&g),
    );
    let mut s = Settings::default();
    s.excluded_games.insert(g.key.encoded());
    let next = engine::build_plan(&p.document, &p.state, &[g], &[], &s).unwrap();
    assert_eq!(next.document, fixture());
}
#[test]
fn user_edited_identity_is_never_overwritten_or_deleted() {
    let g = game("steam", "1", "Game");
    let p = plan(&fixture(), &ManagedState::default(), &[g]);
    let mut edited = p.document;
    edited["apps"][3]["detached"] = json!(["user://custom"]);
    let next = plan(&edited, &p.state, &[]);
    assert_eq!(next.document, edited);
    assert_eq!(next.preview.conflicts.len(), 1);
}
#[test]
fn duplicate_identity_is_ambiguous_and_preserved() {
    let g = game("steam", "1", "Game");
    let p = plan(&fixture(), &ManagedState::default(), &[g]);
    let mut edited = p.document;
    let copy = edited["apps"][3].clone();
    edited["apps"].as_array_mut().unwrap().push(copy);
    let next = plan(&edited, &p.state, &[]);
    assert_eq!(next.document, edited);
    assert_eq!(next.preview.conflicts.len(), 1);
}
#[test]
fn manual_launch_target_is_not_adopted() {
    let g = game("steam", "1", "Game");
    let mut doc = fixture();
    doc["apps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "My manual game", "detached": [g.launch_target.uri]}));
    let p = plan(&doc, &ManagedState::default(), &[g]);
    assert_eq!(p.document, doc);
    assert!(p.state.entries.is_empty());
    assert_eq!(p.preview.conflicts.len(), 1);
}
#[test]
fn updates_preserve_user_prep_commands_and_other_fields() {
    let mut g = game("steam", "1", "Old name");
    let p = plan(&fixture(), &ManagedState::default(), &[g.clone()]);
    let mut edited = p.document;
    edited["apps"][3]["prep-cmd"] = json!([{"do": "custom", "undo": "undo"}]);
    edited["apps"][3]["wait-all"] = false.into();
    g.name = "New name".into();
    let p = plan(&edited, &p.state, &[g]);
    assert_eq!(
        p.document["apps"][3]["prep-cmd"],
        edited["apps"][3]["prep-cmd"]
    );
    assert_eq!(p.document["apps"][3]["wait-all"], false);
    assert_eq!(p.preview.updated.len(), 1);
}
#[test]
fn invalid_apps_json_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    fs::write(&path, b"{broken").unwrap();
    let mut s = Settings::default();
    s.sunshine.apps_path = Some(path.clone());
    assert!(engine::sync(temp.path(), &s, &ScanSnapshot::default(), true, None).is_err());
    assert_eq!(fs::read(path).unwrap(), b"{broken");
}
#[test]
fn transaction_creates_backup_and_noop_preserves_bytes_and_mtime() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    let original = include_bytes!("fixtures/apps.json");
    fs::write(&path, original).unwrap();
    let mut settings = Settings::default();
    settings.sunshine.apps_path = Some(path.clone());
    settings.network.proxy_mode = "direct".into();
    let snapshot = ScanSnapshot {
        games: vec![game("steam", "1", "Game")],
        authoritative_providers: vec!["steam".into()],
        ..Default::default()
    };
    assert!(
        engine::sync(temp.path(), &settings, &snapshot, true, None)
            .unwrap()
            .changed
    );
    assert_eq!(
        fs::read(temp.path().join("apps.json.bak")).unwrap(),
        original
    );
    let before = fs::read(&path).unwrap();
    let time = fs::metadata(&path).unwrap().modified().unwrap();
    assert!(
        !engine::sync(temp.path(), &settings, &snapshot, true, None)
            .unwrap()
            .changed
    );
    assert_eq!(before, fs::read(&path).unwrap());
    assert_eq!(time, fs::metadata(path).unwrap().modified().unwrap());
}
#[test]
fn stale_preview_rejects_external_changes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    storage::write_json(&path, &fixture()).unwrap();
    let mut settings = Settings::default();
    settings.sunshine.apps_path = Some(path.clone());
    let snapshot = ScanSnapshot {
        games: vec![game("steam", "1", "Game")],
        ..Default::default()
    };
    let p = engine::sync(temp.path(), &settings, &snapshot, false, None).unwrap();
    let mut changed = fixture();
    changed["apps"][0]["name"] = "My Desktop".into();
    storage::write_json(&path, &changed).unwrap();
    assert!(engine::sync(
        temp.path(),
        &settings,
        &snapshot,
        true,
        Some(&p.preview.revision)
    )
    .is_err());
    assert_eq!(sunshine::read_apps(&path).unwrap().1, changed);
}
#[test]
fn atomic_write_replaces_without_truncation() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("data");
    storage::atomic_write(&path, b"old").unwrap();
    storage::atomic_write(&path, b"new data").unwrap();
    assert_eq!(fs::read(path).unwrap(), b"new data");
}
#[test]
fn recovery_finishes_committed_state_without_repeating_apps_write() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    let after = b"after";
    fs::write(&path, after).unwrap();
    let s = ManagedState {
        apps_path: Some(path.clone()),
        ..Default::default()
    };
    let j = Journal {
        apps_path: path,
        before: storage::hash(b"before"),
        after: storage::hash(after),
        state: s,
    };
    storage::write_json(&temp.path().join("pending-sync.json"), &j).unwrap();
    state::recover(temp.path()).unwrap();
    assert!(ManagedState::load(temp.path()).unwrap().apps_path.is_some());
    assert!(!temp.path().join("pending-sync.json").exists());
}
#[test]
fn recovery_rejects_ambiguous_external_changes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("apps.json");
    fs::write(&path, b"external").unwrap();
    let j = Journal {
        apps_path: path.clone(),
        before: storage::hash(b"before"),
        after: storage::hash(b"after"),
        state: ManagedState::default(),
    };
    storage::write_json(&temp.path().join("pending-sync.json"), &j).unwrap();
    assert!(state::recover(temp.path()).is_err());
    assert!(!temp.path().join("state.json").exists());
    assert_eq!(fs::read(path).unwrap(), b"external");
}
#[test]
fn invalid_state_never_assumes_ownership() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("state.json"), b"bad").unwrap();
    assert!(ManagedState::load(temp.path()).is_err());
}
#[test]
fn sunshine_file_apps_resolves_relative_to_config_directory() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("config")).unwrap();
    fs::write(
        temp.path().join("config/sunshine.conf"),
        "# file_apps = ignored\nfile_apps = custom.json",
    )
    .unwrap();
    let mut s = Settings::default();
    s.sunshine.install_path = Some(temp.path().into());
    assert_eq!(
        sunshine::apps_path(&s.sunshine).unwrap(),
        temp.path().join("config/custom.json")
    );
}
#[test]
fn settings_roundtrip_allows_future_providers_and_defaults_to_no_writes() {
    let temp = tempfile::tempdir().unwrap();
    let mut s = Settings::default();
    assert!(!s.general.auto_sync);
    s.providers.insert(
        "future".into(),
        ProviderSettings {
            enabled: false,
            path: Some(PathBuf::from("D:/future")),
        },
    );
    s.save(temp.path()).unwrap();
    let loaded = Settings::load(temp.path()).unwrap();
    assert!(!loaded.provider("future").enabled);
    assert!(!loaded.general.auto_sync);
}
#[test]
fn proxy_validation_and_local_sync_do_not_require_network() {
    let mut s = Settings::default();
    s.network.proxy_mode = "custom".into();
    s.network.proxy_url = "invalid".into();
    assert!(s.validate().is_err());
    assert!(NetworkService::new(&s.network).is_err());
    let p = engine::build_plan(
        &fixture(),
        &ManagedState::default(),
        &[game("steam", "1", "Offline")],
        &[],
        &s,
    )
    .unwrap();
    assert_eq!(p.preview.added.len(), 1);
    s.network.proxy_mode = "direct".into();
    assert!(NetworkService::new(&s.network).is_ok());
}

struct BrokenProvider;
impl GameProvider for BrokenProvider {
    fn id(&self) -> &'static str {
        "broken"
    }
    fn display_name(&self) -> &'static str {
        "Broken"
    }
    fn detect(&self) -> anyhow::Result<ProviderDetection> {
        anyhow::bail!("unreadable")
    }
    fn scan_games(&self, _: &ProviderDetection) -> anyhow::Result<ProviderScan> {
        panic!("must not scan failed detection")
    }
    fn launch_command(&self, _: &Game) -> anyhow::Result<LaunchTarget> {
        unreachable!()
    }
    fn artwork_candidates(&self, _: &Game) -> Vec<ArtworkCandidate> {
        vec![]
    }
}
#[test]
fn provider_error_does_not_block_other_providers() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("steamapps")).unwrap();
    let registry: Vec<Box<dyn GameProvider>> = vec![
        Box::new(BrokenProvider),
        Box::new(providers::steam::SteamProvider::new(Some(
            temp.path().into(),
        ))),
    ];
    let result = provider_registry::scan_providers(&registry, &Settings::default());
    assert!(result.providers[0].error.is_some());
    assert!(result.authoritative_providers.contains(&"steam".into()));
}
