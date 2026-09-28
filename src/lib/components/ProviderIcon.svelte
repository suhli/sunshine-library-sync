<script lang="ts">
  import { Gamepad2 } from 'lucide-svelte';
  let { id }: { id: string } = $props();
  // Only icon artwork is provider-specific; layouts are fully data-driven.
  const glyphs: Record<string, string> = { steam: 'S', epic: 'E' };
</script>
<span class="provider-icon" aria-hidden="true">{#if glyphs[id]}<span style="font-weight: 700; font-size: 13px">{glyphs[id]}</span>{:else}<Gamepad2 size={16} />{/if}</span>
