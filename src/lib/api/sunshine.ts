import { call } from './bridge';
import type { SyncPreview, SyncResult, Snapshot } from '$lib/types';
export const getSnapshot = () => call<Snapshot>('get_snapshot');
export const previewSync = () => call<SyncPreview>('get_sync_preview');
export const syncSunshine = (revision: string | null = null, gameKey: string | null = null) => call<SyncResult>('sync_sunshine', { revision, gameKey });
export const restartSunshine = () => call<void>('restart_sunshine');
