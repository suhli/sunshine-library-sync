<script lang="ts">
  import { RefreshCw, FolderOpen } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import ProviderIcon from '$lib/components/ProviderIcon.svelte';
  import Status from '$lib/components/Status.svelte';
  import { providers } from '$lib/stores/providers';
  import { scanning } from '$lib/stores/sync';
  import { refresh, toggleProvider, openLocation, page } from '$lib/stores/app';
  import { desktop } from '$lib/api/bridge';
</script>
<p class="page-intro">Discover installed games from local launcher files. No account access is needed.</p>
{#each $providers as provider}
  <section class="section">
    <div class="section-heading"><div class="inline"><ProviderIcon id={provider.id} /><h2>{provider.display_name}</h2></div><Switch aria-label={`Enable ${provider.display_name}`} checked={provider.enabled} onCheckedChange={value => toggleProvider(provider.id, value)} /></div>
    <div style="margin-bottom: 16px"><Status value={provider.error ? 'error' : provider.detection.installed ? 'installed' : 'not-installed'} />{#if !provider.enabled}<small style="margin-left: 12px">Scanning paused · existing entries kept</small>{/if}</div>
    {#if provider.error}<div class="notice error">{provider.error}</div>{/if}
    {#if provider.detection.installed}
      <dl class="details"><dt>Launcher path</dt><dd class="path">{provider.detection.launcher_path ?? 'Not available'}</dd><dt>Library locations</dt><dd>{provider.detection.data_paths.length}</dd><dt>Installed games</dt><dd>{provider.game_count}</dd></dl>
      <details style="margin-top: 14px"><summary class="muted">Watched locations</summary>{#each provider.detection.data_paths as path}<p class="path" style="margin-top: 6px">{path}</p>{/each}</details>
    {:else}<p class="muted">No local {provider.display_name} installation was detected.</p>{/if}
    {#if provider.warnings.length}<details style="margin-top: 14px"><summary style="color: var(--warning)">{provider.warnings.length} scan warnings · automatic removals paused</summary>{#each provider.warnings as warning}<p class="path" style="margin-top: 8px">{warning}</p>{/each}</details>{/if}
    <div class="actions" style="margin-top: 18px"><Button variant="outline" disabled={$scanning || !provider.enabled} onclick={() => refresh(provider.id)}><RefreshCw size={14} />{provider.error ? 'Retry scan' : 'Scan'}</Button>{#if provider.detection.installed}<Button variant="ghost" onclick={() => openLocation('provider', provider.id)}><FolderOpen size={14} />Open folder</Button>{/if}<Button variant="ghost" onclick={() => $page = 'Settings'}>Configure path</Button></div>
  </section>
{:else}<div class="empty"><div><p>{desktop ? 'Detecting local providers…' : 'Open the desktop app to detect providers.'}</p><small>Installed Steam and Epic libraries will appear here automatically.</small></div></div>{/each}
