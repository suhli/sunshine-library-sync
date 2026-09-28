import { get, writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { desktop } from '$lib/api/bridge';
import * as providerApi from '$lib/api/providers';
import * as sunshineApi from '$lib/api/sunshine';
import * as settingsApi from '$lib/api/settings';
import * as gamesApi from '$lib/api/games';
import type { Settings, Snapshot, SyncResult } from '$lib/types';
import { providers } from './providers';
import { games } from './games';
import { sunshine } from './sunshine';
import { settings } from './settings';
import { syncBusy, scanning, lastSync, preview, previewOpen, appError, paused } from './sync';

export const page = writable<'Overview' | 'Games' | 'Providers' | 'Sunshine' | 'Settings'>('Overview');
export const toast = writable<{ text: string; error: boolean } | null>(null);
let toastTimer: ReturnType<typeof setTimeout>;
export function notify(text: string, error = false) { clearTimeout(toastTimer); toast.set({ text, error }); toastTimer = setTimeout(() => toast.set(null), error ? 12000 : 6000); }
export function report(error: unknown) { notify(error instanceof Error ? error.message : String(error), true); }
export function accept(snapshot: Snapshot) { providers.set(snapshot.scan.providers); games.set(snapshot.scan.games); sunshine.set(snapshot.sunshine); lastSync.set(snapshot.last_sync); preview.set(snapshot.preview); appError.set(snapshot.error); scanning.set(snapshot.scanning); }
function syncMessage(result: SyncResult) {
  const p = result.preview;
  notify(result.reload_error ? `Library saved. ${result.reload_error}` : result.changed ? `Sync completed · ${p.added.length} added, ${p.updated.length} updated, ${p.removed.length} removed` : 'Library is already up to date.', !!result.reload_error);
}
export async function initialize(): Promise<() => void> {
  if (!desktop) { scanning.set(false); return () => {}; }
  const off: UnlistenFn[] = [];
  try {
    off.push(await listen<Snapshot>('library-updated', event => accept(event.payload)));
    off.push(await listen<string>('background-error', event => appError.set(event.payload)));
    off.push(await listen<boolean>('auto-sync-paused', event => paused.set(event.payload)));
    off.push(await listen<SyncResult>('sync-completed', event => { lastSync.set(event.payload); if (!get(syncBusy)) syncMessage(event.payload); }));
    settings.set(await settingsApi.getSettings()); accept(await sunshineApi.getSnapshot());
  } catch (error) { appError.set(String(error)); scanning.set(false); }
  return () => { off.forEach(f => f()); clearTimeout(toastTimer); };
}
export async function refresh(providerId?: string) {
  if (get(scanning) || get(syncBusy)) return;
  scanning.set(true);
  try { accept(await (providerId ? providerApi.scanProvider(providerId) : providerApi.scanAll())); }
  catch (error) { report(error); } finally { scanning.set(false); }
}
export async function showPreview() {
  syncBusy.set(true);
  try { preview.set(await sunshineApi.previewSync()); previewOpen.set(true); } catch (error) { report(error); } finally { syncBusy.set(false); }
}
export async function sync(revision: string | null = null, key: string | null = null) {
  if (get(syncBusy)) return;
  syncBusy.set(true);
  try { const result = await sunshineApi.syncSunshine(revision, key); syncMessage(result); previewOpen.set(false); accept(await sunshineApi.getSnapshot()); }
  catch (error) { report(error); } finally { syncBusy.set(false); }
}
export async function persist(value: Settings) { const saved = await settingsApi.saveSettings(value); settings.set(saved); return saved; }
let settingsOperations: Promise<void> = Promise.resolve();
export async function toggleProvider(id: string, enabled: boolean) {
  settingsOperations = settingsOperations.then(async () => {
    const value = structuredClone(get(settings));
    value.providers[id] = { ...value.providers[id], enabled, path: value.providers[id]?.path ?? null };
    await persist(value);
  }).catch(report);
  await settingsOperations;
}
export async function exclude(key: string, excluded: boolean) {
  settingsOperations = settingsOperations.then(async () => {
    settings.set(await gamesApi.excludeGame(key, excluded));
    notify(excluded ? 'Excluded. Apply Sync to remove its managed entry.' : 'Included in the next sync.');
  }).catch(report);
  await settingsOperations;
}
export async function openLocation(kind: Parameters<typeof gamesApi.openLocation>[0], key?: string) { try { await gamesApi.openLocation(kind, key); } catch (error) { report(error); } }
export async function restart() { syncBusy.set(true); try { await sunshineApi.restartSunshine(); notify('Sunshine restarted.'); accept(await providerApi.scanAll()); } catch (error) { report(error); } finally { syncBusy.set(false); } }
