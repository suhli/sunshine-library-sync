<script lang="ts">
  import { FolderOpen, RefreshCw, ScanLine } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import Status from '$lib/components/Status.svelte';
  import { sunshine } from '$lib/stores/sunshine';
  import { settings } from '$lib/stores/settings';
  import { lastSync, preview, syncBusy } from '$lib/stores/sync';
  import { openLocation, restart, showPreview, sync, page } from '$lib/stores/app';
  import { timestamp } from '$lib/types';
  import { desktop } from '$lib/api/bridge';
</script>
<p class="page-intro">Connection, configuration and sync status for your Sunshine installation.</p>
<section class="section"><div class="section-heading"><h2>Status</h2><Button variant="ghost" onclick={() => $page = 'Settings'}>Edit configuration</Button></div>
  <dl class="details"><dt>Service</dt><dd><Status value={$sunshine.service_status} />{#if $sunshine.service_name}<small style="margin-left: 10px">{$sunshine.service_name}</small>{/if}</dd><dt>Version</dt><dd>{$sunshine.version ?? 'Not available'}</dd><dt>Install path</dt><dd class="path">{$sunshine.install_path ?? 'Not detected'}</dd><dt>apps.json</dt><dd class="path">{$sunshine.apps_path ?? 'Not detected'}</dd></dl>
  {#if $sunshine.error}<p class="notice error" style="margin-top: 16px">{$sunshine.error}</p>{/if}
  <div style="margin-top: 18px"><Button variant="outline" disabled={!desktop || !$sunshine.detected} onclick={() => openLocation('sunshine')}><FolderOpen size={14} />Open config folder</Button></div>
</section>
<section class="section"><div class="section-heading"><h2>Actions</h2></div><div class="actions"><Button variant="outline" disabled={!desktop || !$sunshine.detected || $syncBusy} onclick={() => sync()}><RefreshCw size={14} />Sync Now</Button><Button variant="outline" disabled={!desktop || !$sunshine.service_name || $syncBusy} onclick={restart}>Restart Sunshine</Button></div><p class="muted" style="margin-top: 12px">Restarting Sunshine may interrupt an active stream.</p></section>
<section class="section"><div class="section-heading"><h2>Sync</h2></div><dl class="details"><dt>Managed applications</dt><dd>{$sunshine.managed}</dd><dt>Last sync</dt><dd>{timestamp($lastSync?.completed_at)}</dd><dt>Reload mode</dt><dd>{{ none: 'None · restart Sunshine manually after changes', restart: 'Restart Sunshine service', command: 'Custom command' }[$settings.sunshine.reload_mode]}</dd><dt>Automatic sync</dt><dd>{$settings.general.auto_sync ? 'Enabled' : 'Off'}</dd></dl></section>
<section class="section"><div class="section-heading"><h2>Sync preview</h2></div>{#if $preview}<div class="inline" style="gap: 24px; margin-bottom: 16px"><span>+ {$preview.added.length} added</span><span>~ {$preview.updated.length} changed</span><span>− {$preview.removed.length} removed</span></div>{:else}<p class="muted" style="margin-bottom: 16px">Locate a valid apps.json to preview changes.</p>{/if}<Button variant="outline" disabled={!desktop || !$sunshine.detected || $syncBusy} onclick={showPreview}><ScanLine size={14} />View details</Button></section>
