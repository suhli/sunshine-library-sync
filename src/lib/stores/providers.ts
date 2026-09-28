import { writable } from 'svelte/store';
import type { ProviderInfo } from '$lib/types';
export const providers = writable<ProviderInfo[]>([]);
