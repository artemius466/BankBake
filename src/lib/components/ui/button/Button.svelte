<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends HTMLButtonAttributes {
    variant?: "default" | "secondary" | "outline" | "ghost" | "destructive";
    size?: "default" | "sm" | "lg" | "icon";
    class?: string;
  }

  let {
    variant = "default",
    size = "default",
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variants: Record<NonNullable<Props["variant"]>, string> = {
    default:
      "bg-amber-500 text-zinc-950 font-semibold hover:bg-amber-400 active:bg-amber-500 shadow-sm shadow-amber-500/10",
    secondary:
      "bg-zinc-800/90 text-zinc-100 hover:bg-zinc-700/80 border border-zinc-700/60",
    outline:
      "border border-zinc-800 bg-zinc-900/60 text-zinc-200 hover:bg-zinc-800/80 hover:text-zinc-100 hover:border-zinc-700",
    ghost:
      "text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/60",
    destructive:
      "bg-red-500/15 text-red-400 border border-red-500/30 hover:bg-red-500/25 hover:text-red-300",
  };

  const sizes: Record<NonNullable<Props["size"]>, string> = {
    default: "h-9 px-4 py-2 text-sm rounded-lg",
    sm: "h-7 px-2.5 text-xs rounded-md",
    lg: "h-11 px-6 text-sm rounded-xl",
    icon: "h-8 w-8 rounded-lg flex items-center justify-center p-0",
  };
</script>

<button
  class={cn(
    "inline-flex items-center justify-center gap-2 whitespace-nowrap transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/50 disabled:pointer-events-none disabled:opacity-45 select-none",
    variants[variant],
    sizes[size],
    className
  )}
  {...rest}
>
  {@render children?.()}
</button>
