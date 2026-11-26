<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { GithubReleaseInfo } from "$lib/types";
  import Button from "$lib/components/ui/button/Button.svelte";
  import Switch from "$lib/components/ui/switch/Switch.svelte";
  import { ArrowUpCircle, ExternalLink, X } from "lucide-svelte";

  interface Props {
    openModal: boolean;
    currentVersion: string;
    release: GithubReleaseInfo | null;
    onDismiss: (neverShowAgain: boolean) => void;
  }

  let { openModal, currentVersion, release, onDismiss }: Props = $props();

  let neverShowAgain = $state(false);

  $effect(() => {
    if (openModal) {
      neverShowAgain = false;
    }
  });

  async function handleOpenReleases() {
    const url =
      release?.htmlUrl ||
      "https://github.com/sashaantipov2012/BankBake/releases";
    try {
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
    }
    onDismiss(neverShowAgain);
  }
</script>

{#if openModal && release}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="update-title"
  >
    <div
      class="w-full max-w-md rounded-2xl border border-zinc-800 bg-zinc-900 shadow-2xl overflow-hidden"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between border-b border-zinc-800 px-5 py-3.5"
      >
        <div class="flex items-center gap-2">
          <ArrowUpCircle class="h-4 w-4 text-amber-400" />
          <h2 id="update-title" class="text-sm font-semibold text-zinc-100">
            New Update Available
          </h2>
        </div>
        <button
          type="button"
          onclick={() => onDismiss(neverShowAgain)}
          class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100 transition-colors"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="space-y-3.5 p-5 text-xs">
        <p class="text-zinc-300 leading-relaxed">
          A newer version of <strong class="text-zinc-100">BankBake</strong> is available on GitHub Releases.
        </p>

        <div
          class="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/70 px-3.5 py-2.5 font-mono"
        >
          <div class="text-zinc-400">
            Current: <span class="text-zinc-200">v{currentVersion}</span>
          </div>
          <div class="text-amber-400 font-semibold">
            Latest: v{release.version}
          </div>
        </div>

        {#if release.missedReleases && release.missedReleases.some((r) => r.body && r.body.trim() !== "")}
          <div class="space-y-1">
            <div class="flex items-center justify-between text-[11px] font-medium text-zinc-400">
              <span>Release Notes</span>
              {#if release.missedReleases.length > 1}
                <span class="text-[10px] font-mono text-amber-400/90">
                  {release.missedReleases.length} newer releases
                </span>
              {/if}
            </div>
            <div
              class="max-h-48 overflow-y-auto rounded-lg border border-zinc-800/80 bg-zinc-950/50 p-2.5 text-[11px] text-zinc-300 space-y-3"
            >
              {#each release.missedReleases as item (item.tagName)}
                {#if item.body && item.body.trim() !== ""}
                  <div class="space-y-1 not-first:pt-2.5 not-first:border-t not-first:border-zinc-800/80">
                    <div class="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-amber-400">
                      <span>v{item.version}</span>
                      {#if item.name && item.name !== item.tagName && item.name !== `v${item.version}`}
                        <span class="text-zinc-500 font-sans font-normal">• {item.name}</span>
                      {/if}
                    </div>
                    <div class="whitespace-pre-wrap text-zinc-300 leading-relaxed">{item.body.trim()}</div>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {:else if release.body && release.body.trim() !== ""}
          <div class="space-y-1">
            <div class="text-[11px] font-medium text-zinc-400">Release Notes</div>
            <div
              class="max-h-32 overflow-y-auto rounded-lg border border-zinc-800/80 bg-zinc-950/50 p-2.5 text-[11px] text-zinc-300 whitespace-pre-wrap"
            >
              {release.body.trim()}
            </div>
          </div>
        {/if}

        <!-- Optional "Never show again" toggle -->
        <div
          class="flex items-center justify-between pt-2 border-t border-zinc-800/80"
        >
          <span class="text-zinc-400 select-none">
            Don't show update notifications again
          </span>
          <Switch
            bind:checked={neverShowAgain}
            label="Never show update popup again"
          />
        </div>
      </div>

      <!-- Footer -->
      <div
        class="flex items-center justify-end gap-2 border-t border-zinc-800 bg-zinc-950/50 px-5 py-3"
      >
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onclick={() => onDismiss(neverShowAgain)}
        >
          Skip
        </Button>
        <Button
          type="button"
          variant="default"
          size="sm"
          onclick={handleOpenReleases}
        >
          <ExternalLink class="h-3.5 w-3.5" />
          Open GitHub Releases
        </Button>
      </div>
    </div>
  </div>
{/if}
