import { call } from './bridge';
import type { Settings } from '$lib/types';
export const excludeGame = (gameKey: string, excluded: boolean) => call<Settings>('set_game_excluded', { gameKey, excluded });
export const getArtwork = (gameKey: string) => call<number[]>('get_artwork', { gameKey });
export const openLocation = (kind: 'game' | 'provider' | 'logs' | 'sunshine' | 'data', key: string | null = null) => call<void>('open_path', { kind, key });
