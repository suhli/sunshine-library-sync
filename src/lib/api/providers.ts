import { call } from './bridge';
import type { Snapshot } from '$lib/types';
export const scanAll = () => call<Snapshot>('scan_all_games');
export const scanProvider = (providerId: string) => call<Snapshot>('scan_provider', { providerId });
