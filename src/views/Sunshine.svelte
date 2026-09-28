<script lang="ts">
  import { FolderOpen, RefreshCw, ScanLine } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import Status from '$lib/components/Status.svelte';
  import { sunshine } from '$lib/stores/sunshine';
  import { settings } from '$lib/stores/settings';
  import { lastSync, preview, syncBusy } from '$lib/stores/sync';
  import { openLocation, restart, showPreview, sync, page } from '$lib/stores/app';
  import { t, locale, formatTimestamp } from '$lib/i18n';
  import { desktop } from '$lib/api/bridge';
</script>
<p class="page-intro">{$t('Connection, configuration and sync status for your Sunshine installation.')}</p>
<section class="section"><div class="section-heading"><h2>{$t('Status')}</h2><Button variant="ghost" onclick={() => $page = 'Settings'}>{$t('Edit configuration')}</Button></div>
  <dl class="details"><dt>{$t('Service')}</dt><dd><Status value={$sunshine.service_status} />{#if $sunshine.service_name}<small style="margin-left: 10px">{$sunshine.service_name}</small>{/if}</dd><dt>{$t('Version')}</dt><dd>{$sunshine.version ?? $t('Not available')}</dd><dt>{$t('Install path')}</dt><dd class="path">{$sunshine.install_path ?? $t('Not detected')}</dd><dt>apps.json</dt><dd class="path">{$sunshine.apps_path ?? $t('Not detected')}</dd></dl>
  {#if $sunshine.error}<p class="notice error" style="margin-top: 16px">{$sunshine.error}</p>{/if}
  <div style="margin-top: 18px"><Button variant="outline" disabled={!desktop || !$sunshine.detected} onclick={() => openLocation('sunshine')}><FolderOpen size={14} />{$t('Open config folder')}</Button></div>
</section>
<section class="section"><div class="section-heading"><h2>{$t('Actions')}</h2></div><div class="actions"><Button variant="outline" disabled={!desktop || !$sunshine.detected || $syncBusy} onclick={() => sync()}><RefreshCw size={14} />{$t('Sync Now')}</Button><Button variant="outline" disabled={!desktop || !$sunshine.service_name || $syncBusy} onclick={restart}>{$t('Restart Sunshine')}</Button></div><p class="muted" style="margin-top: 12px">{$t('Restarting Sunshine may interrupt an active stream.')}</p></section>
<section class="section"><div class="section-heading"><h2>{$t('Sync')}</h2></div><dl class="details"><dt>{$t('Managed applications')}</dt><dd>{$sunshine.managed}</dd><dt>{$t('Last sync')}</dt><dd>{formatTimestamp($lastSync?.completed_at, $locale)}</dd><dt>{$t('Reload mode')}</dt><dd>{$t({ none: 'None · restart Sunshine manually after changes', restart: 'Restart Sunshine service', command: 'Custom command' }[$settings.sunshine.reload_mode])}</dd><dt>{$t('Automatic sync')}</dt><dd>{$t($settings.general.auto_sync ? 'Enabled' : 'Off')}</dd></dl></section>
<section class="section"><div class="section-heading"><h2>{$t('Sync preview')}</h2></div>{#if $preview}<div class="inline" style="gap: 24px; margin-bottom: 16px"><span>{$t('+ {count} added', { count: $preview.added.length })}</span><span>{$t('~ {count} changed', { count: $preview.updated.length })}</span><span>{$t('− {count} removed', { count: $preview.removed.length })}</span></div>{:else}<p class="muted" style="margin-bottom: 16px">{$t('Locate a valid apps.json to preview changes.')}</p>{/if}<Button variant="outline" disabled={!desktop || !$sunshine.detected || $syncBusy} onclick={showPreview}><ScanLine size={14} />{$t('View details')}</Button></section>
