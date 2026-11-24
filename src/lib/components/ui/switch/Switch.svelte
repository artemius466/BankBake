<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    checked?: boolean;
    disabled?: boolean;
    id?: string;
    label?: string;
    class?: string;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = $bindable(false),
    disabled = false,
    id,
    label,
    class: className = "",
    onchange,
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {id}
  {disabled}
  onclick={toggle}
  class={cn(
    "peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/50 disabled:cursor-not-allowed disabled:opacity-50",
    checked
      ? "bg-amber-500 border-amber-400/80"
      : "bg-zinc-800 border-zinc-700",
    className
  )}
>
  <span
    class={cn(
      "pointer-events-none block h-3.5 w-3.5 rounded-full shadow-sm transition-transform duration-150",
      checked
        ? "translate-x-4 bg-zinc-950"
        : "translate-x-0.5 bg-zinc-400"
    )}
  ></span>
</button>
