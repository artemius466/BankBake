<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLSpanElement> {
    variant?: "default" | "amber" | "success" | "destructive" | "outline";
    class?: string;
  }

  let {
    variant = "default",
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variants: Record<NonNullable<Props["variant"]>, string> = {
    default: "bg-zinc-800/90 text-zinc-300 border-zinc-700/70",
    amber: "bg-amber-500/15 text-amber-300 border-amber-500/30",
    success: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
    destructive: "bg-red-500/15 text-red-300 border-red-500/30",
    outline: "bg-transparent text-zinc-400 border-zinc-800",
  };
</script>

<span
  class={cn(
    "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium transition-colors",
    variants[variant],
    className
  )}
  {...rest}
>
  {@render children?.()}
</span>
