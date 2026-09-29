export interface GameKey { provider_id: string; provider_game_id: string }
export interface Game { key: GameKey; name: string; install_path: string; manifest_path: string; launch_target: { uri: string }; artwork: string | null; metadata: Record<string, unknown>; sync_status: string }
export interface ProviderInfo { id: string; display_name: string; icon: string; enabled: boolean; detection: { installed: boolean; launcher_path: string | null; data_paths: string[]; version: string | null }; game_count: number; error: string | null; warnings: string[] }
export interface Change { key: string; name: string; provider_id: string }
export interface SyncPreview { added: Change[]; updated: Change[]; removed: Change[]; unchanged: number; warnings: string[]; conflicts: Change[]; revision: string }
export interface SyncResult { preview: SyncPreview; changed: boolean; completed_at: number; reload_error: string | null }
export type SyncAttempt = { status: 'completed'; result: SyncResult } | { status: 'backup_failed'; backup_path: string; reason: string; revision: string };
export interface PendingBackupFailure { backup_path: string; reason: string; revision: string; game_key: string | null }
export interface SunshineStatus { detected: boolean; install_path: string | null; apps_path: string | null; version: string | null; service_name: string | null; service_status: string; applications: number; managed: number; error: string | null }
export interface Snapshot { scan: { providers: ProviderInfo[]; games: Game[]; authoritative_providers: string[]; watch_paths: string[] }; sunshine: SunshineStatus; last_sync: SyncResult | null; preview: SyncPreview | null; error: string | null; scanning: boolean }
export interface Settings {
  general: { auto_sync: boolean; start_with_windows: boolean; start_minimized: boolean; close_behavior: 'tray' | 'exit'; theme: 'system' | 'light' | 'dark'; language: 'system' | 'en' | 'zh-CN' };
  providers: Record<string, { enabled: boolean; path: string | null }>;
  excluded_games: string[];
  sunshine: { install_path: string | null; apps_path: string | null; reload_mode: 'none' | 'restart' | 'command'; reload_command: string };
  network: { proxy_mode: 'system' | 'direct' | 'custom'; proxy_url: string };
  artwork: { provider: 'auto' | 'steam' | 'epic' | 'steamgriddb' | 'local'; steamgriddb_api_key: string };
}
export interface ConnectionTest { connected: boolean; latency_ms: number; message: string }
export const initialSettings: Settings = { general: { auto_sync: false, start_with_windows: false, start_minimized: false, close_behavior: 'tray', theme: 'system', language: 'system' }, providers: {}, excluded_games: [], sunshine: { install_path: null, apps_path: null, reload_mode: 'none', reload_command: '' }, network: { proxy_mode: 'system', proxy_url: '' }, artwork: { provider: 'auto', steamgriddb_api_key: '' } };
export const gameKey = (game: Game) => `${game.key.provider_id}:${game.key.provider_game_id}`;
export const timestamp = (value?: number | null) => value ? new Date(value * 1000).toLocaleString() : 'Never';
