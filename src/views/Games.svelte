<script lang="ts">
  import { Search, Gamepad2, FolderOpen, Copy, Ban, Info, RefreshCw } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import * as ContextMenu from '$lib/components/ui/context-menu';
  import * as Dialog from '$lib/components/ui/dialog';
  import Cover from '$lib/components/Cover.svelte';
  import Status from '$lib/components/Status.svelte';
  import { games } from '$lib/stores/games';
  import { providers } from '$lib/stores/providers';
  import { scanning, syncBusy } from '$lib/stores/sync';
  import { exclude, openLocation, sync, refresh, notify, report } from '$lib/stores/app';
  import { gameKey, type Game } from '$lib/types';
  import { desktop } from '$lib/api/bridge';
  import { t, tr } from '$lib/i18n';
  let search = $state(''); let provider = $state('all'); let status = $state('all');
  let selectedKey = $state<string | null>(null); let detailsOpen = $state(false);
  let selected = $derived($games.find(g => gameKey(g) === selectedKey));
  let filtered = $derived($games.filter(g => g.name.toLowerCase().includes(search.toLowerCase()) && (provider === 'all' || g.key.provider_id === provider) && (status === 'all' || g.sync_status === status)));
  const providerName = (id: string) => $providers.find(p => p.id === id)?.display_name ?? id;
  function select(game: Game, detail = false) { selectedKey = gameKey(game); if (detail) detailsOpen = true; }
  async function copyId() { if (!selected) return; try { await navigator.clipboard.writeText(gameKey(selected)); notify(tr('Game ID copied.')); } catch (error) { report(error); } }
</script>
<div class="toolbar"><div class="search"><Search size={14} /><Input aria-label={$t('Search games')} placeholder={$t('Search games…')} bind:value={search} /></div><select aria-label={$t('Filter by provider')} bind:value={provider}><option value="all">{$t('All providers')}</option>{#each $providers as p}<option value={p.id}>{p.display_name}</option>{/each}</select><select aria-label={$t('Filter by sync status')} bind:value={status}><option value="all">{$t('All statuses')}</option>{#each ['Synced', 'New', 'Changed', 'Excluded', 'Error'] as s}<option value={s.toLowerCase()}>{$t(s)}</option>{/each}</select><span class="total">{$t('{count} games', { count: filtered.length })}</span></div>
<ContextMenu.Root>
  <ContextMenu.Trigger>
    <div class="table-wrap"><table aria-label={$t('Installed games')}><colgroup><col style="width: 48px" /><col style="width: 29%" /><col style="width: 115px" /><col /><col style="width: 94px" /></colgroup><thead><tr><th aria-label={$t('Artwork')}></th><th>{$t('Name')}</th><th>{$t('Provider')}</th><th>{$t('Install path')}</th><th>{$t('Sync status')}</th></tr></thead><tbody>
      {#each filtered as game (gameKey(game))}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <tr class:selected={selectedKey === gameKey(game)} onclick={() => select(game)} ondblclick={() => select(game, true)} oncontextmenu={() => select(game)} onkeydown={event => { if (event.key === 'Enter') select(game, true); }}>
          <td><Cover {game} /></td><td><button class="game-name truncate" title={game.name} onclick={() => select(game)} ondblclick={() => select(game, true)} onkeydown={event => { if (event.key === 'Enter') select(game, true); }}>{game.name}</button></td><td><span class="truncate">{providerName(game.key.provider_id)}</span></td><td class="path truncate" title={game.install_path}>{game.install_path}</td><td><Status value={game.sync_status} /></td>
        </tr>
      {:else}<tr><td colspan="5" style="height: auto"><div class="empty" style="padding: 28px 12px"><Gamepad2 size={22} class="empty-icon" /><div><p>{$t($scanning ? 'Scanning local libraries…' : $games.length ? 'No games match these filters.' : 'No games found.')}</p><small>{$t($games.length ? 'Try another name, provider or status.' : 'Steam and Epic will be scanned automatically in the desktop app.')}</small>{#if $games.length}<Button variant="outline" onclick={() => { search = ''; provider = 'all'; status = 'all'; }}>{$t('Clear filters')}</Button>{:else}<Button variant="outline" disabled={!desktop || $scanning} onclick={() => refresh()}><RefreshCw size={14} />{$t('Scan again')}</Button>{/if}</div></div></td></tr>{/each}
    </tbody></table></div>
  </ContextMenu.Trigger>
  <ContextMenu.Content>
    <ContextMenu.Item disabled={!selected || $syncBusy || selected?.sync_status === 'excluded'} onSelect={() => selected && sync(null, gameKey(selected))}><RefreshCw size={14} />{$t('Sync')}</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}><Ban size={14} />{$t(selected?.sync_status === 'excluded' ? 'Include in Sunshine' : 'Exclude from Sunshine')}</ContextMenu.Item>
    <ContextMenu.Separator />
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && openLocation('game', gameKey(selected))}><FolderOpen size={14} />{$t('Open install directory')}</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={copyId}><Copy size={14} />{$t('Copy game ID')}</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && openLocation('provider', selected.key.provider_id)}>{$t('Open provider folder')}</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => detailsOpen = true}><Info size={14} />{$t('Details')}</ContextMenu.Item>
  </ContextMenu.Content>
</ContextMenu.Root>
<div class="row-actions">{#if selected}<span>{selected.name}</span><div class="actions"><Button size="sm" variant="ghost" onclick={() => detailsOpen = true}>{$t('Details')}</Button><Button size="sm" variant="ghost" onclick={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}>{$t(selected.sync_status === 'excluded' ? 'Include' : 'Exclude')}</Button></div>{:else}<span>{$t('Double-click a game for details. Right-click for actions.')}</span><span>{$t('Local library')}</span>{/if}</div>
<Dialog.Root bind:open={detailsOpen}><Dialog.Content>
  <Dialog.Header><Dialog.Title>{selected?.name ?? $t('Game details')}</Dialog.Title><Dialog.Description>{selected ? providerName(selected.key.provider_id) : $t('Select a game from your library.')}</Dialog.Description></Dialog.Header>
  {#if selected}<dl class="details" style="grid-template-columns: 90px minmax(0, 1fr)"><dt>{$t('Game ID')}</dt><dd class="path">{gameKey(selected)}</dd><dt>{$t('Install')}</dt><dd class="path">{selected.install_path}</dd><dt>{$t('Launch')}</dt><dd class="path">{selected.launch_target.uri}</dd><dt>Sunshine</dt><dd><Status value={selected.sync_status} /></dd><dt>{$t('Artwork')}</dt><dd>{$t(selected.artwork ? 'Cached locally' : 'No cover available')}</dd></dl><Dialog.Footer><Button variant="outline" onclick={() => selected && openLocation('game', gameKey(selected))}><FolderOpen size={14} />{$t('Open folder')}</Button><Button variant="outline" onclick={copyId}><Copy size={14} />{$t('Copy ID')}</Button><Button variant="secondary" onclick={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}>{$t(selected.sync_status === 'excluded' ? 'Include' : 'Exclude')}</Button></Dialog.Footer>{/if}
</Dialog.Content></Dialog.Root>
