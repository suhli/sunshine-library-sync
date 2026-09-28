import { writable } from 'svelte/store';
import type { SyncPreview, SyncResult } from '$lib/types';
export const syncBusy = writable(false);
export const scanning = writable(true);
export const lastSync = writable<SyncResult | null>(null);
export const preview = writable<SyncPreview | null>(null);
export const previewOpen = writable(false);
export const appError = writable<string | null>(null);
export const paused = writable(false);
