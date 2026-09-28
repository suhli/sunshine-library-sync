<script lang="ts">
  import { get } from 'svelte/store';
  import { open } from '@tauri-apps/plugin-dialog';
  import { LoaderCircle, FolderOpen } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Switch } from '$lib/components/ui/switch';
  import { settings } from '$lib/stores/settings';
  import { providers } from '$lib/stores/providers';
  import { persist, notify, report, openLocation } from '$lib/stores/app';
  import { testProxy } from '$lib/api/settings';
  import { desktop } from '$lib/api/bridge';
  import type { ConnectionTest } from '$lib/types';
  let draft = $state(structuredClone(get(settings)));
  let saving = $state(false); let testing = $state(false); let testResult = $state<ConnectionTest | null>(null);
  let dirty = $derived(JSON.stringify(draft) !== JSON.stringify($settings));
  async function save() { saving = true; try { draft = structuredClone(await persist($state.snapshot(draft))); notify('Settings saved.'); } catch (error) { report(error); } finally { saving = false; } }
  async function browse(field: 'apps_path' | 'install_path') { try { const value = await open({ directory: field === 'install_path', multiple: false, title: field === 'apps_path' ? 'Locate Sunshine apps.json' : 'Locate Sunshine folder', filters: field === 'apps_path' ? [{ name: 'JSON configuration', extensions: ['json'] }] : undefined }); if (typeof value === 'string') draft.sunshine[field] = value; } catch (error) { report(error); } }
  async function test() { testing = true; testResult = null; try { testResult = await testProxy($state.snapshot(draft.network)); } catch (error) { testResult = { connected: false, latency_ms: 0, message: String(error) }; } finally { testing = false; } }
  function providerPath(id: string, value: string) { draft.providers[id] = { enabled: draft.providers[id]?.enabled ?? true, path: value.trim() || null }; }
</script>
<div class="section-heading" style="margin-bottom: 24px"><p class="muted">Preferences for this computer.</p><Button variant="ghost" disabled={!desktop} onclick={() => openLocation('data')}><FolderOpen size={14} />Open data folder</Button></div>
<section class="section"><h2>General</h2>
  <div class="setting-row"><div><label class="setting-label" for="theme">Appearance</label><small>Follow Windows or choose a theme.</small></div><div class="control"><select id="theme" bind:value={draft.general.theme}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></div></div>
  <div class="setting-row"><div><span class="setting-label" id="startup-label">Start with Windows</span><small>Start for your Windows account.</small></div><Switch aria-labelledby="startup-label" bind:checked={draft.general.start_with_windows} /></div>
  <div class="setting-row"><div><span class="setting-label" id="minimized-label">Start minimized</span><small>Hide to the tray when started with Windows.</small></div><Switch aria-labelledby="minimized-label" bind:checked={draft.general.start_minimized} /></div>
  <div class="setting-row"><div><label class="setting-label" for="close">Close behavior</label><small>Exit stops all background activity.</small></div><div class="control"><select id="close" bind:value={draft.general.close_behavior}><option value="tray">Minimize to tray</option><option value="exit">Exit application</option></select></div></div>
  <div class="setting-row"><div><span class="setting-label" id="auto-label">Auto Sync</span><small>Sync when installed games change. Off until you enable it.</small></div><Switch aria-labelledby="auto-label" bind:checked={draft.general.auto_sync} /></div>
</section>
<section class="section"><h2>Providers</h2><p class="muted" style="margin: 10px 0">Optional location overrides. Leave blank for automatic detection.</p>
  {#each $providers as p}<div class="setting-row"><div><label class="setting-label" for={`path-${p.id}`}>{p.display_name}</label><small>Launcher folder or manifest location</small></div><div class="control wide"><Input id={`path-${p.id}`} placeholder="Automatic" value={draft.providers[p.id]?.path ?? ''} oninput={event => providerPath(p.id, event.currentTarget.value)} /></div></div>{:else}<small>Provider settings appear after desktop detection.</small>{/each}
</section>
<section class="section"><h2>Artwork</h2>
  <div class="setting-row"><div><label class="setting-label" for="artwork">Artwork source</label><small>Local covers first. Downloads run in the background.</small></div><div class="control"><select id="artwork" bind:value={draft.artwork.provider}><option value="auto">Auto</option><option value="steam">Steam</option><option value="epic">Epic</option><option value="steamgriddb">SteamGridDB</option><option value="local">Local only</option></select></div></div>
  <div class="setting-row"><div><label class="setting-label" for="api-key">SteamGridDB API key</label><small>Optional · saved in your local configuration file.</small></div><div class="control"><Input id="api-key" type="password" autocomplete="off" placeholder="Not configured" bind:value={draft.artwork.steamgriddb_api_key} /></div></div>
</section>
<section class="section"><h2>Network</h2>
  <div class="setting-row"><div><label class="setting-label" for="proxy">Proxy</label><small>Local scans and sync work without a connection.</small></div><div class="control"><select id="proxy" bind:value={draft.network.proxy_mode} onchange={() => testResult = null}><option value="system">System</option><option value="direct">Direct</option><option value="custom">Custom</option></select></div></div>
  {#if draft.network.proxy_mode === 'custom'}<div class="setting-row"><div><label class="setting-label" for="proxy-url">Proxy URL</label><small>HTTP, HTTPS or SOCKS5</small></div><div class="control"><Input id="proxy-url" placeholder="http://127.0.0.1:7897" bind:value={draft.network.proxy_url} oninput={() => testResult = null} /></div></div>{/if}
  <div class="inline" style="margin-top: 12px"><Button variant="outline" disabled={testing || !desktop} onclick={test}>{#if testing}<LoaderCircle size={14} class="spin" />{/if}Test connection</Button>{#if testResult}<span class="status" class:good={testResult.connected} class:bad={!testResult.connected}><span class="status-dot"></span>{testResult.connected ? `Connected · ${testResult.latency_ms} ms` : testResult.message}</span>{/if}</div>
</section>
<section class="section"><h2>Sunshine</h2>
  <div class="setting-row"><div><label class="setting-label" for="sunshine-folder">Install folder</label><small>Optional path override.</small></div><div class="control wide"><Input id="sunshine-folder" placeholder="Automatic" value={draft.sunshine.install_path ?? ''} oninput={event => draft.sunshine.install_path = event.currentTarget.value || null} /><Button variant="outline" disabled={!desktop} onclick={() => browse('install_path')}>Browse</Button></div></div>
  <div class="setting-row"><div><label class="setting-label" for="apps-path">apps.json</label><small>Reads file_apps from sunshine.conf by default.</small></div><div class="control wide"><Input id="apps-path" placeholder="Automatic" value={draft.sunshine.apps_path ?? ''} oninput={event => draft.sunshine.apps_path = event.currentTarget.value || null} /><Button variant="outline" disabled={!desktop} onclick={() => browse('apps_path')}>Browse</Button></div></div>
  <div class="setting-row"><div><label class="setting-label" for="reload">Reload after changes</label><small>Runs once only when apps.json changes.</small></div><div class="control"><select id="reload" bind:value={draft.sunshine.reload_mode}><option value="none">None</option><option value="restart">Restart Sunshine</option><option value="command">Custom command</option></select></div></div>
  {#if draft.sunshine.reload_mode === 'command'}<div class="setting-row"><div><label class="setting-label" for="reload-command">Reload command</label><small>Windows command to run after a successful sync.</small></div><div class="control wide"><Input id="reload-command" bind:value={draft.sunshine.reload_command} /></div></div>{/if}
  {#if draft.sunshine.reload_mode === 'none'}<small>Restart Sunshine manually after syncing to refresh the Moonlight list.</small>{:else if draft.sunshine.reload_mode === 'restart'}<small>Restarting may interrupt a stream and may require administrator rights.</small>{/if}
</section>
<div class="save-bar"><small>{dirty ? 'You have unsaved changes.' : 'Settings are saved locally.'}</small><div class="actions"><Button variant="ghost" disabled={!dirty || saving} onclick={() => draft = structuredClone(get(settings))}>Discard</Button><Button disabled={!dirty || saving || !desktop} onclick={save}>{#if saving}<LoaderCircle size={14} class="spin" />{/if}Save changes</Button></div></div>
