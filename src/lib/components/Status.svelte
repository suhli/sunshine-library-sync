<script lang="ts">
  import { t } from '$lib/i18n';
  let { value, label }: { value: string; label?: string } = $props();
  const labels: Record<string, string> = { running: 'Running', stopped: 'Stopped', 'not-installed': 'Not installed', installed: 'Installed', synced: 'Synced', new: 'New', changed: 'Changed', excluded: 'Excluded', error: 'Error', disabled: 'Disabled', unknown: 'Unknown', pending: 'Starting / stopping' };
  let tone = $derived(['running', 'installed', 'synced'].includes(value) ? 'good' : ['changed', 'pending'].includes(value) ? 'warn' : value === 'error' ? 'bad' : '');
</script>
<span class="status {tone}"><span class="status-dot"></span>{label ?? $t(labels[value] ?? value)}</span>
