<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import WaveSurfer from "wavesurfer.js";
  import type { AudioEventCardItem } from "$lib/types";
  import { formatDuration, formatBytes } from "$lib/utils";
  import Input from "$lib/components/ui/input/Input.svelte";
  import Switch from "$lib/components/ui/switch/Switch.svelte";
  import {
    Play,
    Pause,
    Square,
    Repeat,
    ChevronDown,
    ChevronUp,
    Trash2,
    Volume2,
    Activity,
    Music,
    Sparkles,
    FileAudio,
  } from "lucide-svelte";

  interface Props {
    item: AudioEventCardItem;
    onRemove: (id: string) => void;
    onUpdate: (item: AudioEventCardItem) => void;
  }

  let { item = $bindable(), onRemove, onUpdate }: Props = $props();

  let isPlaying = $state(false);
  let currentTime = $state(0);
  let loadingAudio = $state(false);
  let audioError = $state<string | null>(null);

  let htmlAudio: HTMLAudioElement | null = null;
  let wavesurfer: WaveSurfer | null = null;
  let waveformContainer = $state<HTMLDivElement | null>(null);
  let blobUrl: string | null = null;
  let animFrameId: number | null = null;

  function mimeFromExt(filename: string): string {
    const ext = filename.split(".").pop()?.toLowerCase() ?? "";
    if (ext === "ogg") return "audio/ogg";
    if (ext === "mp3") return "audio/mpeg";
    if (ext === "flac") return "audio/flac";
    if (ext === "aif" || ext === "aiff") return "audio/aiff";
    return "audio/wav";
  }

  async function ensureBlobUrl(): Promise<string> {
    if (blobUrl) return blobUrl;
    loadingAudio = true;
    audioError = null;
    try {
      const bytes = await invoke<number[]>("read_audio_bytes", {
        path: item.filePath,
      });
      const uint8 = new Uint8Array(bytes);
      const blob = new Blob([uint8], { type: mimeFromExt(item.filename) });
      blobUrl = URL.createObjectURL(blob);
      return blobUrl;
    } catch (e: any) {
      audioError = typeof e === "string" ? e : e?.message ?? "Failed to load audio";
      throw e;
    } finally {
      loadingAudio = false;
    }
  }

  function dbToLinearGain(db: number): number {
    // Clamp between 0 and 1 for HTMLAudioElement preview
    const gain = Math.pow(10, db / 20);
    return Math.max(0, Math.min(1, gain));
  }

  async function initHtmlAudio() {
    if (htmlAudio) return htmlAudio;
    const url = await ensureBlobUrl();
    const audio = new Audio(url);
    audio.volume = dbToLinearGain(item.volumeDb);

    audio.addEventListener("loadedmetadata", () => {
      if (audio.duration && Number.isFinite(audio.duration) && audio.duration > 0) {
        item.duration = audio.duration;
        if (item.loopEnd === null || item.loopEnd > audio.duration) {
          item.loopEnd = Number(audio.duration.toFixed(2));
        }
        onUpdate(item);
      }
    });

    audio.addEventListener("ended", () => {
      if (item.looping) {
        const start = item.customLoopEnabled ? Math.max(0, item.loopStart) : 0;
        audio.currentTime = Math.min(start, audio.duration || 0);
        audio.play().catch(() => {});
      } else {
        isPlaying = false;
        currentTime = 0;
      }
    });

    htmlAudio = audio;
    return audio;
  }

  function monitorPlayback() {
    if (wavesurfer && item.showWaveform) {
      currentTime = wavesurfer.getCurrentTime();
      const dur = wavesurfer.getDuration() || item.duration;
      if (item.looping && item.customLoopEnabled && item.loopEnd && item.loopEnd > item.loopStart) {
        if (currentTime >= item.loopEnd) {
          wavesurfer.setTime(item.loopStart);
        }
      } else if (item.looping && dur > 0 && currentTime >= dur - 0.03) {
        wavesurfer.setTime(0);
      }
    } else if (htmlAudio) {
      currentTime = htmlAudio.currentTime;
      if (
        item.looping &&
        item.customLoopEnabled &&
        item.loopEnd &&
        item.loopEnd > item.loopStart &&
        currentTime >= item.loopEnd
      ) {
        htmlAudio.currentTime = item.loopStart;
      }
    }

    if (isPlaying) {
      animFrameId = requestAnimationFrame(monitorPlayback);
    }
  }

  async function togglePlay() {
    try {
      if (item.showWaveform) {
        if (!wavesurfer) {
          await initWaveSurfer();
        }
        if (wavesurfer) {
          wavesurfer.setVolume(dbToLinearGain(item.volumeDb));
          await wavesurfer.playPause();
          isPlaying = wavesurfer.isPlaying();
          if (isPlaying) {
            monitorPlayback();
          }
          return;
        }
      }

      const audio = await initHtmlAudio();
      audio.volume = dbToLinearGain(item.volumeDb);
      if (isPlaying) {
        audio.pause();
        isPlaying = false;
      } else {
        if (
          item.looping &&
          item.customLoopEnabled &&
          (audio.currentTime < item.loopStart ||
            (item.loopEnd && audio.currentTime >= item.loopEnd))
        ) {
          audio.currentTime = item.loopStart;
        }
        await audio.play();
        isPlaying = true;
        monitorPlayback();
      }
    } catch (err) {
      isPlaying = false;
    }
  }

  function stopPlayback() {
    if (wavesurfer) {
      wavesurfer.stop();
    }
    if (htmlAudio) {
      htmlAudio.pause();
      htmlAudio.currentTime = 0;
    }
    isPlaying = false;
    currentTime = 0;
    if (animFrameId) {
      cancelAnimationFrame(animFrameId);
      animFrameId = null;
    }
  }

  async function initWaveSurfer() {
    if (!waveformContainer || wavesurfer) return;
    const url = await ensureBlobUrl();
    if (!waveformContainer) return;

    const ws = WaveSurfer.create({
      container: waveformContainer,
      waveColor: "#52525b",
      progressColor: "#f59e0b",
      cursorColor: "#fbbf24",
      barWidth: 2,
      barGap: 1.5,
      barRadius: 2,
      height: 44,
      normalize: true,
      url,
    });

    ws.on("ready", () => {
      const dur = ws.getDuration();
      if (dur && Number.isFinite(dur) && dur > 0) {
        item.duration = dur;
        if (item.loopEnd === null || item.loopEnd > dur) {
          item.loopEnd = Number(dur.toFixed(2));
        }
        onUpdate(item);
      }
    });

    ws.on("play", () => {
      isPlaying = true;
      monitorPlayback();
    });

    ws.on("pause", () => {
      isPlaying = false;
    });

    ws.on("finish", () => {
      if (item.looping) {
        const start = item.customLoopEnabled ? Math.max(0, item.loopStart) : 0;
        ws.setTime(start);
        ws.play();
      } else {
        isPlaying = false;
        currentTime = 0;
      }
    });

    ws.on("timeupdate", (t) => {
      currentTime = t;
    });

    wavesurfer = ws;
  }

  async function toggleWaveform() {
    stopPlayback();
    item.showWaveform = !item.showWaveform;
    onUpdate(item);
    if (item.showWaveform) {
      // Wait a microtask for container to mount
      setTimeout(() => {
        initWaveSurfer();
      }, 20);
    } else if (wavesurfer) {
      wavesurfer.destroy();
      wavesurfer = null;
    }
  }

  function setCategory(cat: "SFX" | "MUSIC") {
    item.category = cat;
    const trimmed = item.eventName.trim();
    if (cat === "MUSIC") {
      if (/^event:\/sfx(\/|$)/i.test(trimmed)) {
        item.eventName = trimmed.replace(/^event:\/sfx(\/|$)/i, "event:/music$1");
      } else if (/^\/?sfx(\/|$)/i.test(trimmed)) {
        item.eventName = trimmed.replace(/^(\/?)sfx(\/|$)/i, "$1music$2");
      }
    } else {
      if (/^event:\/music(\/|$)/i.test(trimmed)) {
        item.eventName = trimmed.replace(/^event:\/music(\/|$)/i, "event:/sfx$1");
      } else if (/^\/?music(\/|$)/i.test(trimmed)) {
        item.eventName = trimmed.replace(/^(\/?)music(\/|$)/i, "$1sfx$2");
      }
    }
    onUpdate(item);
  }

  function handleVolumeInput(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    item.volumeDb = Number.isFinite(val) ? val : 0;
    const gain = dbToLinearGain(item.volumeDb);
    if (htmlAudio) htmlAudio.volume = gain;
    if (wavesurfer) wavesurfer.setVolume(gain);
    onUpdate(item);
  }

  onMount(() => {
    // Probe duration quietly if not already probed by backend
    if (!item.duration || item.duration <= 0) {
      initHtmlAudio().catch(() => {});
    }
  });

  onDestroy(() => {
    stopPlayback();
    if (wavesurfer) {
      wavesurfer.destroy();
      wavesurfer = null;
    }
    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
      blobUrl = null;
    }
  });
</script>

<div
  class="group rounded-xl border border-zinc-800/85 bg-zinc-900/60 hover:border-zinc-700/90 transition-all duration-150 shadow-sm"
>
  <!-- Main Compact Row -->
  <div class="flex flex-wrap items-center gap-3 px-3.5 py-2.5">
    <!-- Play / Pause Button -->
    <button
      type="button"
      onclick={togglePlay}
      disabled={loadingAudio}
      title={isPlaying ? "Pause preview" : "Preview audio"}
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border transition-all duration-150 {isPlaying
        ? 'bg-amber-500 text-zinc-950 border-amber-400 shadow-sm shadow-amber-500/20'
        : 'bg-zinc-800/90 text-zinc-200 border-zinc-700/70 hover:bg-zinc-700/80 hover:text-amber-300'}"
    >
      {#if isPlaying}
        <Pause class="h-3.5 w-3.5 fill-current" />
      {:else}
        <Play class="h-3.5 w-3.5 fill-current ml-0.5" />
      {/if}
    </button>

    {#if isPlaying || currentTime > 0}
      <button
        type="button"
        onclick={stopPlayback}
        title="Stop preview"
        class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-zinc-800/70 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-700/70 transition-colors"
      >
        <Square class="h-3 w-3 fill-current" />
      </button>
    {/if}

    <!-- Audio Filename & Event Name Input -->
    <div class="min-w-[180px] flex-1 grid grid-cols-1 sm:grid-cols-2 gap-2.5 items-center">
      <div class="flex items-center gap-2 min-w-0">
        <FileAudio class="h-4 w-4 shrink-0 text-amber-400/80" />
        <div class="min-w-0 flex-1">
          <div
            class="truncate text-xs font-medium text-zinc-300"
            title={item.filePath}
          >
            {item.filename}
          </div>
          <div class="flex items-center gap-2 text-[11px] text-zinc-500 font-mono">
            <span>
              {#if isPlaying || currentTime > 0}
                <span class="text-amber-400">{formatDuration(currentTime)}</span> /
              {/if}
              {formatDuration(item.duration)}
            </span>
            <span>•</span>
            <span>{formatBytes(item.sizeBytes)}</span>
          </div>
        </div>
      </div>

      <!-- Event Path Input (Fully Editable, supports nested paths like event:/sfx/player/jump) -->
      <div class="flex items-center">
        <Input
          bind:value={item.eventName}
          oninput={() => onUpdate(item)}
          placeholder={item.category === "MUSIC"
            ? "event:/music/track_name"
            : "event:/sfx/player/sound_name"}
          aria-label="Event path"
          class="h-7 px-2.5 py-1 text-xs font-mono bg-zinc-950/70 border-zinc-800 focus-visible:border-amber-500/70"
        />
      </div>
    </div>

    <!-- Music / SFX Segmented Selection -->
    <div
      class="inline-flex h-7 shrink-0 items-center rounded-lg border border-zinc-800 bg-zinc-950/80 p-0.5"
      role="group"
      aria-label="Event category"
    >
      <button
        type="button"
        onclick={() => setCategory("SFX")}
        class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-medium transition-all {item.category ===
        'SFX'
          ? 'bg-amber-500 text-zinc-950 shadow-sm font-semibold'
          : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Sparkles class="h-3 w-3" />
        SFX
      </button>
      <button
        type="button"
        onclick={() => setCategory("MUSIC")}
        class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-medium transition-all {item.category ===
        'MUSIC'
          ? 'bg-amber-500 text-zinc-950 shadow-sm font-semibold'
          : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <Music class="h-3 w-3" />
        Music
      </button>
    </div>

    <!-- Looping Toggle -->
    <div
      class="flex items-center gap-1.5 shrink-0 px-1.5 py-1 rounded-lg border border-transparent hover:border-zinc-800/80"
      title="Enable timeline loop region"
    >
      <Repeat
        class="h-3.5 w-3.5 transition-colors {item.looping
          ? 'text-amber-400'
          : 'text-zinc-500'}"
      />
      <span
        class="text-xs select-none {item.looping
          ? 'text-amber-300 font-medium'
          : 'text-zinc-400'}"
      >
        Loop
      </span>
      <Switch
        bind:checked={item.looping}
        label="Toggle looping"
        onchange={() => onUpdate(item)}
      />
    </div>

    <!-- Waveform Preview Toggle -->
    <button
      type="button"
      onclick={toggleWaveform}
      title="Toggle waveform visualization"
      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border transition-colors {item.showWaveform
        ? 'bg-amber-500/15 border-amber-500/40 text-amber-300'
        : 'border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
    >
      <Activity class="h-3.5 w-3.5" />
    </button>

    <!-- Collapsible Advanced Settings Toggle -->
    <button
      type="button"
      onclick={() => {
        item.showAdvanced = !item.showAdvanced;
        onUpdate(item);
      }}
      title="Advanced volume and loop settings"
      class="inline-flex h-7 shrink-0 items-center gap-1 rounded-lg border px-2 text-xs transition-colors {item.showAdvanced ||
      item.volumeDb !== 0 ||
      (item.looping && item.customLoopEnabled)
        ? 'bg-amber-500/10 border-amber-500/35 text-amber-300'
        : 'border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'}"
    >
      <span>Advanced</span>
      {#if item.showAdvanced}
        <ChevronUp class="h-3.5 w-3.5" />
      {:else}
        <ChevronDown class="h-3.5 w-3.5" />
      {/if}
    </button>

    <!-- Remove Event Button -->
    <button
      type="button"
      onclick={() => onRemove(item.id)}
      title="Remove event"
      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-zinc-500 hover:bg-red-500/15 hover:text-red-400 transition-colors"
    >
      <Trash2 class="h-3.5 w-3.5" />
    </button>
  </div>

  <!-- Optional WaveSurfer.js Waveform Drawer -->
  {#if item.showWaveform}
    <div class="border-t border-zinc-800/70 bg-zinc-950/60 px-4 py-2.5">
      {#if audioError}
        <div class="text-xs text-red-400 py-1">{audioError}</div>
      {/if}
      <div
        bind:this={waveformContainer}
        class="w-full rounded-md overflow-hidden cursor-pointer"
      ></div>
    </div>
  {/if}

  <!-- Collapsible Advanced Settings (Hidden by Default) -->
  {#if item.showAdvanced}
    <div
      class="border-t border-zinc-800/70 bg-zinc-950/45 px-4 py-3 text-xs grid grid-cols-1 md:grid-cols-2 gap-4 rounded-b-xl"
    >
      <!-- Volume Control (dB) -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <label
            for="vol-{item.id}"
            class="flex items-center gap-1.5 font-medium text-zinc-300"
          >
            <Volume2 class="h-3.5 w-3.5 text-amber-400" />
            <span>Track Volume</span>
          </label>
          <div class="flex items-center gap-2">
            <span class="font-mono text-amber-300">
              {item.volumeDb > 0 ? `+${item.volumeDb.toFixed(1)}` : item.volumeDb.toFixed(1)} dB
            </span>
            {#if item.volumeDb !== 0}
              <button
                type="button"
                onclick={() => {
                  item.volumeDb = 0;
                  if (htmlAudio) htmlAudio.volume = 1;
                  if (wavesurfer) wavesurfer.setVolume(1);
                  onUpdate(item);
                }}
                class="text-[11px] text-zinc-400 hover:text-amber-300 underline"
              >
                Reset
              </button>
            {/if}
          </div>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-[11px] font-mono text-zinc-500">-24</span>
          <input
            id="vol-{item.id}"
            type="range"
            min="-24"
            max="10"
            step="0.5"
            value={item.volumeDb}
            oninput={handleVolumeInput}
            class="h-1.5 w-full cursor-pointer appearance-none rounded-lg bg-zinc-800 accent-amber-500"
          />
          <span class="text-[11px] font-mono text-zinc-500">+10</span>
        </div>
      </div>

      <!-- Custom Loop Points -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="font-medium text-zinc-300">Custom Loop Region (sec)</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-[11px] text-zinc-400">Enable custom points</span>
            <Switch
              bind:checked={item.customLoopEnabled}
              disabled={!item.looping}
              label="Enable custom loop points"
              onchange={() => {
                if (item.customLoopEnabled && item.loopEnd === null && item.duration > 0) {
                  item.loopEnd = Number(item.duration.toFixed(2));
                }
                onUpdate(item);
              }}
            />
          </div>
        </div>

        {#if !item.looping}
          <p class="text-[11px] text-zinc-500 italic">
            Enable the "Loop" toggle on this event card to configure custom loop points.
          </p>
        {:else if !item.customLoopEnabled}
          <p class="text-[11px] text-zinc-500">
            Looping full audio duration (0.00s → {item.duration > 0 ? item.duration.toFixed(2) : "end"}s).
          </p>
        {:else}
          <div class="grid grid-cols-2 gap-2.5 pt-0.5">
            <div>
              <label
                for="lstart-{item.id}"
                class="block text-[11px] text-zinc-400 mb-1"
              >
                Loop Start (s)
              </label>
              <Input
                id="lstart-{item.id}"
                type="number"
                min="0"
                step="0.05"
                bind:value={item.loopStart}
                oninput={() => onUpdate(item)}
                class="h-7 text-xs font-mono bg-zinc-900"
              />
            </div>
            <div>
              <label
                for="lend-{item.id}"
                class="block text-[11px] text-zinc-400 mb-1"
              >
                Loop End (s)
              </label>
              <Input
                id="lend-{item.id}"
                type="number"
                min="0.05"
                step="0.05"
                bind:value={item.loopEnd}
                placeholder={item.duration > 0 ? item.duration.toFixed(2) : "End"}
                oninput={() => onUpdate(item)}
                class="h-7 text-xs font-mono bg-zinc-900"
              />
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
