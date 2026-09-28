import { call } from './bridge';
import type { Settings, ConnectionTest } from '$lib/types';
export const getSettings = () => call<Settings>('get_settings');
export const saveSettings = (settings: Settings) => call<Settings>('save_settings', { settings });
export const testProxy = (network: Settings['network']) => call<ConnectionTest>('test_proxy', { network });
