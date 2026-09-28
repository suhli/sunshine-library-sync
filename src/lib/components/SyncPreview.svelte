<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { preview, previewOpen, syncBusy } from '$lib/stores/sync';
  import { sync, showPreview } from '$lib/stores/app';
  import { LoaderCircle } from 'lucide-svelte';
  import { t } from '$lib/i18n';
</script>
<Dialog.Root bind:open={$previewOpen}>
  <Dialog.Content>
    <Dialog.Header><Dialog.Title>{$t('Sync preview')}</Dialog.Title><Dialog.Description>{$t('Review changes to applications managed by Library Sync.')}</Dialog.Description></Dialog.Header>
    {#if $preview}
      <div class="dialog-scroll">
        {#each [{ label: 'Added', items: $preview.added }, { label: 'Updated', items: $preview.updated }, { label: 'Removed', items: $preview.removed }] as group}
          <section class="dialog-group"><h3>{$t(group.label)} <span class="muted">{group.items.length}</span></h3>
            {#each group.items as item}<div class="recent-row"><span class:removed={group.label === 'Removed'} class="change-symbol">{group.label === 'Added' ? '+' : group.label === 'Removed' ? '−' : '~'}</span>{item.name}<small>{item.provider_id}</small></div>{:else}<small>{$t('No changes')}</small>{/each}
          </section>
        {/each}
        {#each $preview.warnings as warning}<p class="notice">{warning}</p>{/each}
      </div>
      <small>{$t('{count} unchanged · Manual Sunshine applications are preserved.', { count: $preview.unchanged })}</small>
    {/if}
    <Dialog.Footer><Button variant="ghost" onclick={showPreview} disabled={$syncBusy}>{$t('Refresh preview')}</Button><Button variant="outline" onclick={() => $previewOpen = false}>{$t('Cancel')}</Button><Button disabled={$syncBusy || !$preview} onclick={() => sync($preview?.revision ?? null)}>{#if $syncBusy}<LoaderCircle size={14} class="spin" />{/if}{$t('Apply Sync')}</Button></Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
