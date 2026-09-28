import { call } from './bridge';
import type { Settings, ConnectionTest } from '$lib/types';
export const getSettings = () => call<Settings>('get_settings');
export const saveSettings = (settings: Settings) => call<Settings>('save_settings', { settings });
export const testProxy = (network: Settings['network']) => call<ConnectionTest>('test_proxy', { network });
export const getRuntimeInfo = () => call<{ system_locale: string; fixture_mode: boolean }>('get_runtime_info');
