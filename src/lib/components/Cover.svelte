<script lang="ts">
  import { Gamepad2 } from 'lucide-svelte';
  import { getArtwork } from '$lib/api/games';
  import { gameKey, type Game } from '$lib/types';
  let { game }: { game: Game } = $props();
  let src = $state<string | null>(null);
  $effect(() => {
    const key = gameKey(game); const artwork = game.artwork; let alive = true; let url: string | null = null;
    if (artwork) getArtwork(key).then(data => { if (alive) { url = URL.createObjectURL(new Blob([new Uint8Array(data)], { type: 'image/png' })); src = url; } }).catch(() => {});
    return () => { alive = false; if (url) URL.revokeObjectURL(url); src = null; };
  });
</script>
<span class="cover">{#if src}<img src={src} alt="" loading="lazy" onerror={() => src = null} />{:else}<Gamepad2 size={15} />{/if}</span>
