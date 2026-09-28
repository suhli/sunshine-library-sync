import { writable } from 'svelte/store';
import type { Game } from '$lib/types';
export const games = writable<Game[]>([]);
