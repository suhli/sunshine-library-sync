<script lang="ts">
  import { onMount } from 'svelte';
  import { LayoutDashboard, Library, Boxes, Sun, Settings2, RefreshCw, Ellipsis, FolderOpen, Power, LoaderCircle, Info, Check, X } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import SyncPreview from '$lib/components/SyncPreview.svelte';
  import Status from '$lib/components/Status.svelte';
  import Overview from './views/Overview.svelte';
  import Games from './views/Games.svelte';
  import Providers from './views/Providers.svelte';
  import Sunshine from './views/Sunshine.svelte';
  import Settings from './views/Settings.svelte';
  import { page, toast, initialize, sync, refresh, restart, openLocation, showPreview } from '$lib/stores/app';
  import { scanning, syncBusy, appError, paused } from '$lib/stores/sync';
  import { sunshine } from '$lib/stores/sunshine';
  import { settings } from '$lib/stores/settings';
  import { desktop } from '$lib/api/bridge';
  const navigation = [{ label: 'Overview' as const, icon: LayoutDashboard }, { label: 'Games' as const, icon: Library }, { label: 'Providers' as const, icon: Boxes }, { label: 'Sunshine' as const, icon: Sun }, { label: 'Settings' as const, icon: Settings2 }];
  let systemDark = $state(false);
  onMount(() => { let cleanup = () => {}; let alive = true; initialize().then(fn => { if (alive) cleanup = fn; else fn(); }); const query = matchMedia('(prefers-color-scheme: dark)'); systemDark = query.matches; const listener = (e: MediaQueryListEvent) => systemDark = e.matches; query.addEventListener('change', listener); return () => { alive = false; cleanup(); query.removeEventListener('change', listener); }; });
  $effect(() => { document.documentElement.classList.toggle('dark', $settings.general.theme === 'dark' || ($settings.general.theme === 'system' && systemDark)); });
</script>
<div class="app-shell">
  <aside class="sidebar"><div class="brand"><div class="brand-mark"><Sun size={20} strokeWidth={1.5} /></div><div><strong>Sunshine</strong><small>Library Sync</small></div></div><nav aria-label="Main navigation">{#each navigation as item}<button class="nav-item" class:active={$page === item.label} aria-current={$page === item.label ? 'page' : undefined} onclick={() => $page = item.label}><item.icon size={17} strokeWidth={1.7} />{item.label}</button>{/each}</nav><div class="sidebar-footer"><small>Local games. Simply synced.</small><br /><small>Version 0.1.0</small></div></aside>
  <div class="workspace"><header class="app-header"><h1>{$page}</h1><div class="header-actions"><Button disabled={!desktop || $syncBusy || $scanning || !$sunshine.detected} onclick={() => sync()}>{#if $syncBusy}<LoaderCircle size={14} class="spin" />Syncing…{:else}<RefreshCw size={14} />Sync Now{/if}</Button><DropdownMenu.Root><DropdownMenu.Trigger aria-label="More actions">{#snippet child({ props })}<Button {...props} variant="ghost" size="icon"><Ellipsis size={18} /></Button>{/snippet}</DropdownMenu.Trigger><DropdownMenu.Content align="end"><DropdownMenu.Item disabled={!desktop || $scanning} onSelect={() => refresh()}><RefreshCw size={14} />Refresh</DropdownMenu.Item><DropdownMenu.Item disabled={!desktop || !$sunshine.detected || $syncBusy} onSelect={showPreview}>Preview Sync</DropdownMenu.Item><DropdownMenu.Separator /><DropdownMenu.Item disabled={!desktop || !$sunshine.service_name || $syncBusy} onSelect={restart}><Power size={14} />Restart Sunshine</DropdownMenu.Item><DropdownMenu.Item disabled={!desktop} onSelect={() => openLocation('logs')}><FolderOpen size={14} />Open Logs</DropdownMenu.Item></DropdownMenu.Content></DropdownMenu.Root></div></header>
    <main class="content">
      {#if !desktop}<div class="notice"><Info size={16} /><div class="message"><strong>Desktop preview</strong><br /><small>Open the Windows app to detect local games and connect Sunshine.</small></div></div>{/if}
      {#if $appError}<div class="notice error" role="alert"><Info size={16} /><div class="message">{$appError}</div><Button variant="ghost" size="icon" aria-label="Dismiss error" onclick={() => $appError = null}><X size={14} /></Button></div>{/if}
      {#if $page === 'Overview'}<Overview />{:else if $page === 'Games'}<Games />{:else if $page === 'Providers'}<Providers />{:else if $page === 'Sunshine'}<Sunshine />{:else}<Settings />{/if}
    </main>
  </div>
  <footer class="statusbar"><Status value={$sunshine.service_status} label={`Sunshine ${$sunshine.service_status === 'running' ? 'running' : $sunshine.detected ? 'detected' : 'not detected'}`} /><span>{$scanning ? 'Scanning libraries…' : $syncBusy ? 'Syncing…' : 'Ready'}</span><span class="right">{$paused ? 'Auto Sync paused' : $settings.general.auto_sync ? 'Auto Sync on' : 'Auto Sync off'}</span></footer>
</div>
<SyncPreview />
{#if $toast}<div class="toast" class:error={$toast.error} role={$toast.error ? 'alert' : 'status'}>{#if $toast.error}<Info size={16} />{:else}<Check size={16} />{/if}<span style="flex: 1">{$toast.text}</span><button aria-label="Dismiss notification" onclick={() => $toast = null}><X size={14} /></button></div>{/if}
