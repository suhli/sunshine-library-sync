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
  let search = $state(''); let provider = $state('all'); let status = $state('all');
  let selectedKey = $state<string | null>(null); let detailsOpen = $state(false);
  let selected = $derived($games.find(g => gameKey(g) === selectedKey));
  let filtered = $derived($games.filter(g => g.name.toLowerCase().includes(search.toLowerCase()) && (provider === 'all' || g.key.provider_id === provider) && (status === 'all' || g.sync_status === status)));
  const providerName = (id: string) => $providers.find(p => p.id === id)?.display_name ?? id;
  function select(game: Game, detail = false) { selectedKey = gameKey(game); if (detail) detailsOpen = true; }
  async function copyId() { if (!selected) return; try { await navigator.clipboard.writeText(gameKey(selected)); notify('Game ID copied.'); } catch (error) { report(error); } }
</script>
<div class="toolbar"><div class="search"><Search size={14} /><Input aria-label="Search games" placeholder="Search games…" bind:value={search} /></div><select aria-label="Filter by provider" bind:value={provider}><option value="all">All providers</option>{#each $providers as p}<option value={p.id}>{p.display_name}</option>{/each}</select><select aria-label="Filter by sync status" bind:value={status}><option value="all">All statuses</option>{#each ['Synced', 'New', 'Changed', 'Excluded', 'Error'] as s}<option value={s.toLowerCase()}>{s}</option>{/each}</select><span class="total">{filtered.length} games</span></div>
<ContextMenu.Root>
  <ContextMenu.Trigger>
    <div class="table-wrap"><table aria-label="Installed games"><colgroup><col style="width: 48px" /><col style="width: 29%" /><col style="width: 115px" /><col /><col style="width: 94px" /></colgroup><thead><tr><th aria-label="Artwork"></th><th>Name</th><th>Provider</th><th>Install path</th><th>Sync status</th></tr></thead><tbody>
      {#each filtered as game (gameKey(game))}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <tr class:selected={selectedKey === gameKey(game)} onclick={() => select(game)} ondblclick={() => select(game, true)} oncontextmenu={() => select(game)} onkeydown={event => { if (event.key === 'Enter') select(game, true); }}>
          <td><Cover {game} /></td><td><button class="game-name truncate" title={game.name} onclick={() => select(game)} ondblclick={() => select(game, true)} onkeydown={event => { if (event.key === 'Enter') select(game, true); }}>{game.name}</button></td><td><span class="truncate">{providerName(game.key.provider_id)}</span></td><td class="path truncate" title={game.install_path}>{game.install_path}</td><td><Status value={game.sync_status} /></td>
        </tr>
      {:else}<tr><td colspan="5" style="height: auto"><div class="empty" style="padding: 28px 12px"><Gamepad2 size={22} class="empty-icon" /><div><p>{$scanning ? 'Scanning local libraries…' : $games.length ? 'No games match these filters.' : 'No games found.'}</p><small>{$games.length ? 'Try another name, provider or status.' : 'Steam and Epic will be scanned automatically in the desktop app.'}</small>{#if $games.length}<Button variant="outline" onclick={() => { search = ''; provider = 'all'; status = 'all'; }}>Clear filters</Button>{:else}<Button variant="outline" disabled={!desktop || $scanning} onclick={() => refresh()}><RefreshCw size={14} />Scan again</Button>{/if}</div></div></td></tr>{/each}
    </tbody></table></div>
  </ContextMenu.Trigger>
  <ContextMenu.Content>
    <ContextMenu.Item disabled={!selected || $syncBusy || selected?.sync_status === 'excluded'} onSelect={() => selected && sync(null, gameKey(selected))}><RefreshCw size={14} />Sync</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}><Ban size={14} />{selected?.sync_status === 'excluded' ? 'Include in Sunshine' : 'Exclude from Sunshine'}</ContextMenu.Item>
    <ContextMenu.Separator />
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && openLocation('game', gameKey(selected))}><FolderOpen size={14} />Open install directory</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={copyId}><Copy size={14} />Copy game ID</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => selected && openLocation('provider', selected.key.provider_id)}>Open provider folder</ContextMenu.Item>
    <ContextMenu.Item disabled={!selected} onSelect={() => detailsOpen = true}><Info size={14} />Details</ContextMenu.Item>
  </ContextMenu.Content>
</ContextMenu.Root>
<div class="row-actions">{#if selected}<span>{selected.name}</span><div class="actions"><Button size="sm" variant="ghost" onclick={() => detailsOpen = true}>Details</Button><Button size="sm" variant="ghost" onclick={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}>{selected.sync_status === 'excluded' ? 'Include' : 'Exclude'}</Button></div>{:else}<span>Double-click a game for details. Right-click for actions.</span><span>Local library</span>{/if}</div>
<Dialog.Root bind:open={detailsOpen}><Dialog.Content>
  <Dialog.Header><Dialog.Title>{selected?.name ?? 'Game details'}</Dialog.Title><Dialog.Description>{selected ? providerName(selected.key.provider_id) : 'Select a game from your library.'}</Dialog.Description></Dialog.Header>
  {#if selected}<dl class="details" style="grid-template-columns: 90px minmax(0, 1fr)"><dt>Game ID</dt><dd class="path">{gameKey(selected)}</dd><dt>Install</dt><dd class="path">{selected.install_path}</dd><dt>Launch</dt><dd class="path">{selected.launch_target.uri}</dd><dt>Sunshine</dt><dd><Status value={selected.sync_status} /></dd><dt>Artwork</dt><dd>{selected.artwork ? 'Cached locally' : 'No cover available'}</dd></dl><Dialog.Footer><Button variant="outline" onclick={() => selected && openLocation('game', gameKey(selected))}><FolderOpen size={14} />Open folder</Button><Button variant="outline" onclick={copyId}><Copy size={14} />Copy ID</Button><Button variant="secondary" onclick={() => selected && exclude(gameKey(selected), selected.sync_status !== 'excluded')}>{selected.sync_status === 'excluded' ? 'Include' : 'Exclude'}</Button></Dialog.Footer>{/if}
</Dialog.Content></Dialog.Root>
