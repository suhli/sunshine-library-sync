import { writable } from 'svelte/store';
import { initialSettings } from '$lib/types';
export const settings = writable(structuredClone(initialSettings));
