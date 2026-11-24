<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends HTMLInputAttributes {
    value?: string | number | null;
    class?: string;
  }

  let {
    value = $bindable(""),
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<input
  bind:value
  class={cn(
    "flex h-9 w-full rounded-lg border border-zinc-800 bg-zinc-900/80 px-3 py-1.5 text-sm text-zinc-100 shadow-inner transition-colors placeholder:text-zinc-500 focus-visible:outline-none focus-visible:border-amber-500/70 focus-visible:ring-1 focus-visible:ring-amber-500/40 disabled:cursor-not-allowed disabled:opacity-50",
    className
  )}
  {...rest}
/>
