<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { permissionFailure, permissionPromptOpen, syncBusy } from '$lib/stores/sync';
  import { dismissPermissionFailure, restartElevated } from '$lib/stores/app';
  import { t } from '$lib/i18n';
</script>

{#if $permissionFailure}
  <Dialog.Root bind:open={$permissionPromptOpen}>
    <Dialog.Content showCloseButton={false}>
      <Dialog.Header>
        <Dialog.Title>{$t('Administrator permission required')}</Dialog.Title>
        <Dialog.Description>{$t('Windows denied access to Sunshine apps.json. The file was not changed.')}</Dialog.Description>
      </Dialog.Header>
      <div class="notice error backup-reason" role="alert">{$permissionFailure.reason}</div>
      <p class="muted backup-path">{$t('Target file')}: <code>{$permissionFailure.path}</code></p>
      {#if $permissionFailure.backup_path}
        <p class="muted backup-path">{$t('Backup destination')}: <code>{$permissionFailure.backup_path}</code></p>
      {/if}
      <p>{$t('Restart with administrator permission, then review and apply the sync again.')}</p>
      <Dialog.Footer>
        <Button variant="outline" disabled={$syncBusy} onclick={dismissPermissionFailure}>{$t('Cancel sync')}</Button>
        <Button disabled={$syncBusy} onclick={restartElevated}>{$t('Restart as administrator')}</Button>
      </Dialog.Footer>
    </Dialog.Content>
  </Dialog.Root>
{/if}
