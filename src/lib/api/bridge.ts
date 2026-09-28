import { invoke, isTauri } from '@tauri-apps/api/core';
export const desktop = isTauri();
export function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!desktop) return Promise.reject(new Error('Open the desktop application to access local games and Sunshine.'));
  return invoke<T>(command, args);
}
