use crate::{
    config::Settings,
    models::*,
    providers::{self, GameProvider},
};
use std::collections::BTreeSet;

pub fn scan(settings: &Settings) -> ScanSnapshot {
    scan_providers(&providers::registry(settings), settings)
}
pub fn scan_providers(registry: &[Box<dyn GameProvider>], settings: &Settings) -> ScanSnapshot {
    let mut snapshot = ScanSnapshot::default();
    for provider in registry {
        let enabled = settings.provider(provider.id()).enabled;
        let mut info = ProviderInfo {
            id: provider.id().into(),
            display_name: provider.display_name().into(),
            icon: provider.id().into(),
            enabled,
            detection: ProviderDetection::default(),
            game_count: 0,
            error: None,
            warnings: vec![],
        };
        match provider.detect() {
            Err(e) => {
                info.error = Some(format!("Unable to detect {}: {e}", provider.display_name()))
            }
            Ok(detection) => {
                if enabled {
                    snapshot
                        .watch_paths
                        .extend(provider.watch_paths(&detection));
                    if detection.installed {
                        match provider.scan_games(&detection) {
                            Ok(result) => {
                                if result.complete {
                                    snapshot.authoritative_providers.push(provider.id().into());
                                }
                                info.game_count = result.games.len();
                                info.warnings = result.warnings;
                                snapshot.games.extend(result.games);
                            }
                            Err(e) => info.error = Some(format!("Scan failed: {e}")),
                        }
                    }
                    // A missing launcher/disk does not prove that its games were
                    // uninstalled. Preserve managed entries until a complete scan.
                }
                info.detection = detection;
            }
        }
        tracing::info!(
            "[Provider:{}] Found {} games",
            info.display_name,
            info.game_count
        );
        snapshot.providers.push(info);
    }
    let mut seen = BTreeSet::new();
    snapshot.games.retain(|game| seen.insert(game.key.clone()));
    snapshot
        .games
        .sort_by_key(|g| (g.name.to_lowercase(), g.key.clone()));
    snapshot.watch_paths.sort();
    snapshot.watch_paths.dedup();
    snapshot
}
