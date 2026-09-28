import { writable } from 'svelte/store';
import type { SunshineStatus } from '$lib/types';
export const sunshine = writable<SunshineStatus>({ detected: false, install_path: null, apps_path: null, version: null, service_name: null, service_status: 'not-installed', applications: 0, managed: 0, error: null });
