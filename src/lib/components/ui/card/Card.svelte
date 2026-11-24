<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    class?: string;
  }

  let { class: className = "", children, ...rest }: Props = $props();
</script>

<div
  class={cn(
    "rounded-xl border border-zinc-800/80 bg-zinc-900/55 text-zinc-100 shadow-sm backdrop-blur-sm transition-colors",
    className
  )}
  {...rest}
>
  {@render children?.()}
</div>
