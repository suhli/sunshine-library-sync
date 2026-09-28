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
  import { t, locale, formatTimestamp } from '$lib/i18n';
  import { desktop } from '$lib/api/bridge';
  let changes = $derived($lastSync ? [...$lastSync.preview.added.map(c => ({ ...c, symbol: '+' })), ...$lastSync.preview.updated.map(c => ({ ...c, symbol: '~' })), ...$lastSync.preview.removed.map(c => ({ ...c, symbol: '−' }))] : []);
</script>
<p class="page-intro">{$t('Your installed games, connected to Sunshine.')}</p>
<section class="section">
  <div class="section-heading"><h2>Sunshine</h2><Button variant="ghost" size="sm" onclick={() => $page = 'Sunshine'}>{$t('View details')} <ArrowRight size={14} /></Button></div>
  {#if $scanning && !$sunshine.detected}<Skeleton class="h-5 w-44 mb-4" /><Skeleton class="h-4 w-80" />
  {:else if $sunshine.detected}
    <div class="inline" style="margin-bottom: 18px"><Status value={$sunshine.service_status} /><small>{$sunshine.version ? $t('Version {version}', { version: $sunshine.version }) : $t('Local installation detected')}</small></div>
    <dl class="details"><dt>{$t('Applications')}</dt><dd>{$t('{count} total', { count: $sunshine.applications })} <span class="muted">{$t('· {count} managed', { count: $sunshine.managed })}</span></dd><dt>{$t('Last sync')}</dt><dd>{formatTimestamp($lastSync?.completed_at, $locale)}</dd><dt>{$t('Configuration')}</dt><dd class="path truncate" title={$sunshine.apps_path ?? ''}>{$sunshine.apps_path}</dd></dl>
  {:else}<div class="empty"><FolderSearch size={21} class="empty-icon" /><div><p>{$t('Sunshine was not detected.')}</p><small>{$t('Locate its apps.json file to connect your library.')}</small><Button variant="outline" onclick={() => $page = 'Settings'}>{$t('Locate Sunshine')}</Button></div></div>{/if}
</section>
<section class="section">
  <div class="section-heading"><h2>{$t('Providers')}</h2><Button variant="ghost" size="sm" onclick={() => $page = 'Providers'}>{$t('Manage')} <ArrowRight size={14} /></Button></div>
  {#each $providers as provider}
    <div class="provider-row"><ProviderIcon id={provider.id} /><span class="name">{provider.display_name}</span><Status value={provider.error ? 'error' : provider.detection.installed ? 'installed' : 'not-installed'} /><span class="count">{$t('{count} games', { count: provider.game_count })}</span><Switch aria-label={$t('Enable {name}', { name: provider.display_name })} checked={provider.enabled} onCheckedChange={value => toggleProvider(provider.id, value)} /></div>
  {:else}{#if $scanning}<Skeleton class="h-10 w-full mb-2" /><Skeleton class="h-10 w-full" />{:else}<div class="empty"><Gamepad2 size={21} class="empty-icon" /><div><p>{$t(desktop ? 'No providers available.' : 'Local providers are available in the desktop app.')}</p><small>{$t('Steam and Epic are discovered from their local installation files.')}</small></div></div>{/if}{/each}
</section>
<section class="section">
  <div class="section-heading"><h2>{$t('Library changes')}</h2>{#if $preview}<Button variant="ghost" size="sm" onclick={showPreview}>{$t('Preview sync')} <ArrowRight size={14} /></Button>{/if}</div>
  {#if $preview && ($preview.added.length + $preview.updated.length + $preview.removed.length > 0)}
    <div class="inline" style="gap: 24px; margin-bottom: 16px"><span>{$t('+ {count} added', { count: $preview.added.length })}</span><span>{$t('~ {count} changed', { count: $preview.updated.length })}</span><span>{$t('− {count} removed', { count: $preview.removed.length })}</span></div>
    <small>{$t('Ready to sync. Review the changes or choose Sync Now.')}</small>
  {:else if !$lastSync}<p class="muted">{$t('No sync yet. Your first scan leaves Sunshine unchanged.')}</p>
  {:else}<p class="muted">{$t($preview?.warnings.length ? 'Some entries need attention. Open the sync preview for details.' : 'Your library is up to date.')}</p>{/if}
</section>
<section class="section">
  <div class="section-heading"><h2>{$t('Last sync')}</h2><small>{formatTimestamp($lastSync?.completed_at, $locale)}</small></div>
  {#each changes.slice(0, 6) as change}<div class="recent-row"><span class="change-symbol" class:removed={change.symbol === '−'}>{change.symbol}</span>{change.name}<small>{change.provider_id}</small></div>{:else}<small>{$t('Changes will appear here after your first sync.')}</small>{/each}
</section>
<Button variant="outline" disabled={!desktop || $scanning} onclick={() => refresh()}><RefreshCw size={14} class={$scanning ? 'spin' : ''} />{$t('Scan again')}</Button>
