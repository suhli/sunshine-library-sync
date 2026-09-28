<script lang="ts">
  import { ArrowRight, FolderSearch, Gamepad2, RefreshCw } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import Status from '$lib/components/Status.svelte';
  import ProviderIcon from '$lib/components/ProviderIcon.svelte';
  import { providers } from '$lib/stores/providers';
  import { sunshine } from '$lib/stores/sunshine';
  import { scanning, lastSync, preview } from '$lib/stores/sync';
  import { page, refresh, showPreview, toggleProvider } from '$lib/stores/app';
  import { timestamp } from '$lib/types';
  import { desktop } from '$lib/api/bridge';
  let changes = $derived($lastSync ? [...$lastSync.preview.added.map(c => ({ ...c, symbol: '+' })), ...$lastSync.preview.updated.map(c => ({ ...c, symbol: '~' })), ...$lastSync.preview.removed.map(c => ({ ...c, symbol: '−' }))] : []);
</script>
<p class="page-intro">Your installed games, connected to Sunshine.</p>
<section class="section">
  <div class="section-heading"><h2>Sunshine</h2><Button variant="ghost" size="sm" onclick={() => $page = 'Sunshine'}>View details <ArrowRight size={14} /></Button></div>
  {#if $scanning && !$sunshine.detected}<Skeleton class="h-5 w-44 mb-4" /><Skeleton class="h-4 w-80" />
  {:else if $sunshine.detected}
    <div class="inline" style="margin-bottom: 18px"><Status value={$sunshine.service_status} /><small>{$sunshine.version ? `Version ${$sunshine.version}` : 'Local installation detected'}</small></div>
    <dl class="details"><dt>Applications</dt><dd>{$sunshine.applications} total <span class="muted">· {$sunshine.managed} managed</span></dd><dt>Last sync</dt><dd>{timestamp($lastSync?.completed_at)}</dd><dt>Configuration</dt><dd class="path truncate" title={$sunshine.apps_path ?? ''}>{$sunshine.apps_path}</dd></dl>
  {:else}<div class="empty"><FolderSearch size={21} class="empty-icon" /><div><p>Sunshine was not detected.</p><small>Locate its apps.json file to connect your library.</small><Button variant="outline" onclick={() => $page = 'Settings'}>Locate Sunshine</Button></div></div>{/if}
</section>
<section class="section">
  <div class="section-heading"><h2>Providers</h2><Button variant="ghost" size="sm" onclick={() => $page = 'Providers'}>Manage <ArrowRight size={14} /></Button></div>
  {#each $providers as provider}
    <div class="provider-row"><ProviderIcon id={provider.id} /><span class="name">{provider.display_name}</span><Status value={provider.error ? 'error' : provider.detection.installed ? 'installed' : 'not-installed'} /><span class="count">{provider.game_count} games</span><Switch aria-label={`Enable ${provider.display_name}`} checked={provider.enabled} onCheckedChange={value => toggleProvider(provider.id, value)} /></div>
  {:else}{#if $scanning}<Skeleton class="h-10 w-full mb-2" /><Skeleton class="h-10 w-full" />{:else}<div class="empty"><Gamepad2 size={21} class="empty-icon" /><div><p>{desktop ? 'No providers available.' : 'Local providers are available in the desktop app.'}</p><small>Steam and Epic are discovered from their local installation files.</small></div></div>{/if}{/each}
</section>
<section class="section">
  <div class="section-heading"><h2>Library changes</h2>{#if $preview}<Button variant="ghost" size="sm" onclick={showPreview}>Preview sync <ArrowRight size={14} /></Button>{/if}</div>
  {#if $preview && ($preview.added.length + $preview.updated.length + $preview.removed.length > 0)}
    <div class="inline" style="gap: 24px; margin-bottom: 16px"><span>+ {$preview.added.length} added</span><span>~ {$preview.updated.length} changed</span><span>− {$preview.removed.length} removed</span></div>
    <small>Ready to sync. Review the changes or choose Sync Now.</small>
  {:else if !$lastSync}<p class="muted">No sync yet. Your first scan leaves Sunshine unchanged.</p>
  {:else}<p class="muted">{$preview?.warnings.length ? 'Some entries need attention. Open the sync preview for details.' : 'Your library is up to date.'}</p>{/if}
</section>
<section class="section">
  <div class="section-heading"><h2>Last sync</h2><small>{timestamp($lastSync?.completed_at)}</small></div>
  {#each changes.slice(0, 6) as change}<div class="recent-row"><span class="change-symbol" class:removed={change.symbol === '−'}>{change.symbol}</span>{change.name}<small>{change.provider_id}</small></div>{:else}<small>Changes will appear here after your first sync.</small>{/each}
</section>
<Button variant="outline" disabled={!desktop || $scanning} onclick={() => refresh()}><RefreshCw size={14} class={$scanning ? 'spin' : ''} />Scan again</Button>
