import { call } from './bridge';
import type { SyncAttempt, SyncPreview, Snapshot } from '$lib/types';
export const getSnapshot = () => call<Snapshot>('get_snapshot');
export const previewSync = () => call<SyncPreview>('get_sync_preview');
export const syncSunshine = (revision: string | null = null, gameKey: string | null = null, allowWithoutBackup = false) => call<SyncAttempt>('sync_sunshine', { revision, gameKey, allowWithoutBackup });
export const restartAsAdmin = () => call<void>('restart_as_admin');
export const restartSunshine = () => call<void>('restart_sunshine');
