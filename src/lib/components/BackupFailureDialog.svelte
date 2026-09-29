<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { backupFailure, backupPromptOpen, syncBusy } from '$lib/stores/sync';
  import { continueWithoutBackup, dismissBackupFailure } from '$lib/stores/app';
  import { t } from '$lib/i18n';
</script>

{#if $backupFailure}
  <Dialog.Root bind:open={$backupPromptOpen}>
    <Dialog.Content showCloseButton={false}>
      <Dialog.Header>
        <Dialog.Title>{$t('Backup failed')}</Dialog.Title>
        <Dialog.Description>{$t('Sunshine apps.json was not changed. Choose whether to continue this sync without a backup.')}</Dialog.Description>
      </Dialog.Header>
      <div class="notice error backup-reason" role="alert">{$backupFailure.reason}</div>
      <p class="muted backup-path">{$t('Backup destination')}: <code>{$backupFailure.backup_path}</code></p>
      <p>{$t('Continuing will update apps.json without a backup from this sync. Cancel keeps it unchanged.')}</p>
      <Dialog.Footer>
        <Button variant="outline" disabled={$syncBusy} onclick={dismissBackupFailure}>{$t('Cancel sync')}</Button>
        <Button variant="destructive" disabled={$syncBusy} onclick={continueWithoutBackup}>{$t('Continue without backup')}</Button>
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
{/if}
