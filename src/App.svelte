<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import type {
    AppConfig,
    AudioEventCardItem,
    AudioFileInfo,
    BakeBankRequest,
    BakeBankResult,
    CompilerInfo,
  } from "$lib/types";
  import { formatBytes } from "$lib/utils";
  import Button from "$lib/components/ui/button/Button.svelte";
  import Input from "$lib/components/ui/input/Input.svelte";
  import Badge from "$lib/components/ui/badge/Badge.svelte";
  import EventCard from "$lib/components/EventCard.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import {
    Flame,
    Settings,
    UploadCloud,
    FolderPlus,
    Trash2,
    CheckCircle2,
    AlertCircle,
    Loader2,
    FolderOpen,
    Sparkles,
    ChevronDown,
    ChevronUp,
    FileCode2,
    Disc3,
  } from "lucide-svelte";

  // Application State
  let bankName = $state("SoundBank");
  let outputDirInput = $state("./BakedBanks");
  let events = $state<AudioEventCardItem[]>([]);
  let isDragging = $state(false);
  let settingsOpen = $state(false);

  let config = $state<AppConfig>({
    mode: "standalone",
    fmodCompilerPath: null,
    outputDirectory: null,
    buildMasterBanks: true,
    exportGuids: true,
    keepTempProject: false,
    isDebugBuild: import.meta.env.DEV,
  });

  let isDebugBuild = $derived(
    config.isDebugBuild !== undefined
      ? config.isDebugBuild
      : import.meta.env.DEV
  );

  let compilerInfo = $state<CompilerInfo>({
    available: false,
    path: null,
    source: "not_found",
    version: null,
    message: "Checking FMOD Studio compiler...",
  });

  let buildStatus = $state<"idle" | "baking" | "success" | "error">("idle");
  let buildError = $state<string | null>(null);
  let lastBakeResult = $state<BakeBankResult | null>(null);
  let showBuildDetails = $state(false);

  let unlistenDragDrop: (() => void) | null = null;

  async function refreshCompilerStatus(customPath?: string | null) {
    try {
      const info = await invoke<CompilerInfo>("check_fmod_compiler", {
        customPath: customPath ?? null,
      });
      compilerInfo = info;
    } catch (e: any) {
      compilerInfo = {
        available: false,
        path: null,
        source: "not_found",
        version: null,
        message: typeof e === "string" ? e : "Failed to query compiler",
      };
    }
  }

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>("get_app_config");
      config = {
        ...loaded,
        mode: "standalone",
      };
      outputDirInput = loaded.outputDirectory ?? "./BakedBanks";
    } catch {
      // keep defaults
    }
  }

  async function handleSaveConfig(newCfg: AppConfig) {
    const trimmedOut = outputDirInput.trim();
    const merged: AppConfig = {
      ...newCfg,
      mode: "standalone",
      outputDirectory: trimmedOut !== "" ? trimmedOut : null,
    };
    config = await invoke<AppConfig>("save_app_config", {
      newConfig: merged,
    });
    outputDirInput = config.outputDirectory ?? "./BakedBanks";
    await refreshCompilerStatus(config.fmodCompilerPath);
  }

  async function persistOutputDirectory(dirValue: string) {
    const trimmed = dirValue.trim();
    const updated: AppConfig = {
      ...config,
      mode: "standalone",
      outputDirectory: trimmed !== "" ? trimmed : null,
    };
    config = updated;
    try {
      config = await invoke<AppConfig>("save_app_config", {
        newConfig: updated,
      });
    } catch {
      // ignore persistence failure
    }
  }

  async function pickOutputFolder() {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        defaultPath:
          outputDirInput && !outputDirInput.startsWith(".")
            ? outputDirInput
            : undefined,
      });
      if (selected && typeof selected === "string") {
        outputDirInput = selected;
        await persistOutputDirectory(selected);
      }
    } catch {
      // dialog cancelled
    }
  }

  function guessCategory(info: AudioFileInfo): "SFX" | "MUSIC" {
    const lowerName = info.filename.toLowerCase();
    const lowerPath = info.path.toLowerCase();
    if (
      lowerName.includes("music") ||
      lowerName.includes("bgm") ||
      lowerName.includes("ost") ||
      lowerPath.includes("\\music\\") ||
      lowerPath.includes("/music/") ||
      (info.durationSeconds && info.durationSeconds >= 25)
    ) {
      return "MUSIC";
    }
    return "SFX";
  }

  async function addAudioPaths(paths: string[]) {
    if (!paths || paths.length === 0) return;
    buildError = null;
    try {
      const inspected = await invoke<AudioFileInfo[]>("inspect_audio_files", {
        paths,
      });

      const existingPaths = new Set(events.map((e) => e.filePath.toLowerCase()));
      const existingNames = new Set(events.map((e) => e.eventName.toLowerCase()));

      const added: AudioEventCardItem[] = [];
      for (const info of inspected) {
        if (existingPaths.has(info.path.toLowerCase())) {
          continue;
        }
        existingPaths.add(info.path.toLowerCase());

        const cat = guessCategory(info);
        const prefix = cat === "MUSIC" ? "event:/music/" : "event:/sfx/";
        let evName = `${prefix}${info.suggestedEventName}`;
        let suffix = 2;
        while (existingNames.has(evName.toLowerCase())) {
          evName = `${prefix}${info.suggestedEventName}_${suffix}`;
          suffix++;
        }
        existingNames.add(evName.toLowerCase());

        const dur = info.durationSeconds ?? 0;

        added.push({
          id: crypto.randomUUID(),
          filePath: info.path,
          filename: info.filename,
          eventName: evName,
          category: cat,
          looping: cat === "MUSIC",
          volumeDb: 0,
          customLoopEnabled: false,
          loopStart: 0,
          loopEnd: dur > 0 ? Number(dur.toFixed(2)) : null,
          duration: dur,
          sizeBytes: info.sizeBytes,
          showWaveform: false,
          showAdvanced: false,
        });
      }

      if (added.length > 0) {
        events = [...events, ...added];
      }
    } catch (e: any) {
      buildStatus = "error";
      buildError = typeof e === "string" ? e : e?.message ?? "Failed to import audio files.";
    }
  }

  async function browseAudioFiles() {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
        filters: [
          {
            name: "Audio Files",
            extensions: ["wav", "ogg", "mp3", "flac", "aif", "aiff", "wma"],
          },
        ],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      await addAudioPaths(paths);
    } catch {
      // dialog cancelled
    }
  }

  async function loadBundledSampleFiles() {
    buildError = null;
    try {
      const samples = await invoke<AudioFileInfo[]>("list_sample_audio_files");
      if (samples.length > 0) {
        await addAudioPaths(samples.map((s) => s.path));
      }
    } catch (e: any) {
      buildStatus = "error";
      buildError = typeof e === "string" ? e : "Could not load sample audio files.";
    }
  }

  function removeEvent(id: string) {
    events = events.filter((e) => e.id !== id);
  }

  function clearAllEvents() {
    events = [];
    if (buildStatus !== "baking") {
      buildStatus = "idle";
      buildError = null;
    }
  }

  function updateEvent(updated: AudioEventCardItem) {
    events = events.map((e) => (e.id === updated.id ? updated : e));
  }

  async function handleBakeBank() {
    if (buildStatus === "baking") return;
    const trimmedBank = bankName.trim();
    if (!trimmedBank) {
      buildStatus = "error";
      buildError = "Please enter a valid bank name.";
      return;
    }
    if (events.length === 0) {
      buildStatus = "error";
      buildError = "Please import at least one audio file before baking.";
      return;
    }

    buildStatus = "baking";
    buildError = null;
    lastBakeResult = null;

    const trimmedOutDir = outputDirInput.trim();
    if (trimmedOutDir !== (config.outputDirectory ?? "")) {
      await persistOutputDirectory(trimmedOutDir);
    }

    const payload: BakeBankRequest = {
      bankName: trimmedBank,
      mode: "standalone",
      events: events.map((e) => ({
        id: e.id,
        filePath: e.filePath,
        filename: e.filename,
        eventName:
          e.eventName.trim() ||
          (e.category === "MUSIC"
            ? "event:/music/unnamed_event"
            : "event:/sfx/unnamed_event"),
        category: e.category,
        looping: e.looping,
        volumeDb: Number(e.volumeDb) || 0,
        customLoopEnabled: e.customLoopEnabled,
        loopStart: Number(e.loopStart) || 0,
        loopEnd:
          e.loopEnd !== null && e.loopEnd !== undefined && Number(e.loopEnd) > 0
            ? Number(e.loopEnd)
            : null,
        duration: Number(e.duration) || 1.0,
      })),
      outputDirectory: trimmedOutDir !== "" ? trimmedOutDir : null,
      fmodCompilerPath: config.fmodCompilerPath,
      buildMasterBanks: config.buildMasterBanks,
      exportGuids: config.exportGuids,
    };

    try {
      const result = await invoke<BakeBankResult>("bake_bank", {
        request: payload,
      });
      lastBakeResult = result;
      buildStatus = "success";
    } catch (e: any) {
      buildStatus = "error";
      buildError = typeof e === "string" ? e : e?.message ?? "Bank compilation failed.";
    }
  }

  async function openOutputFolder() {
    const targetFile =
      lastBakeResult?.builtFiles?.[0]?.path ??
      config.outputDirectory;
    if (!targetFile) return;
    try {
      await revealItemInDir(targetFile);
    } catch {
      // fallback if reveal fails
    }
  }

  onMount(async () => {
    await loadConfig();
    await refreshCompilerStatus();

    // Tauri v2 native window file drag-and-drop listener
    try {
      const webview = getCurrentWebview();
      unlistenDragDrop = await webview.onDragDropEvent((event) => {
        if (event.payload.type === "over" || event.payload.type === "enter") {
          isDragging = true;
        } else if (event.payload.type === "drop") {
          isDragging = false;
          if (event.payload.paths && event.payload.paths.length > 0) {
            addAudioPaths(event.payload.paths);
          }
        } else {
          isDragging = false;
        }
      });
    } catch {
      // Fallback when running in standard browser preview
    }
  });

  onDestroy(() => {
    if (unlistenDragDrop) {
      unlistenDragDrop();
      unlistenDragDrop = null;
    }
  });
</script>

<div class="min-h-screen flex flex-col bg-zinc-950 text-zinc-100">
  <!-- Simple Header with App Name, Compiler Indicator & Settings Button -->
  <header
    class="sticky top-0 z-30 border-b border-zinc-800/80 bg-zinc-950/85 backdrop-blur-md"
  >
    <div
      class="mx-auto flex max-w-4xl items-center justify-between px-5 py-3"
    >
      <!-- App Brand -->
      <div class="flex items-center gap-2.5">
        <div
          class="flex h-8 w-8 items-center justify-center rounded-xl bg-amber-500/15 border border-amber-500/35 text-amber-400 shadow-xs"
        >
          <Flame class="h-4 w-4 fill-amber-400/25" />
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h1 class="text-sm font-semibold tracking-tight text-zinc-100">
              BankBake
            </h1>
            <span
              class="rounded-md bg-zinc-900 border border-zinc-800 px-1.5 py-0.5 text-[10px] font-mono text-zinc-400"
            >
              FMOD 2.02
            </span>
          </div>
        </div>
      </div>

      <!-- Right Controls: Compiler Availability Indicator (Debug or when Missing) + Settings Button -->
      <div class="flex items-center gap-2.5">
        {#if isDebugBuild || !compilerInfo.available}
          <button
            type="button"
            onclick={() => (settingsOpen = true)}
            title={compilerInfo.path ?? compilerInfo.message}
            class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium transition-colors {compilerInfo.available
              ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/15'
              : 'border-amber-500/35 bg-amber-500/10 text-amber-300 hover:bg-amber-500/15'}"
          >
            <span
              class="h-2 w-2 rounded-full {compilerInfo.available
                ? 'bg-emerald-400'
                : 'bg-amber-400 animate-pulse'}"
            ></span>
            <span>
              {compilerInfo.available
                ? "FMOD CLI Ready"
                : "Compiler Not Found"}
            </span>
          </button>
        {/if}

        <Button
          type="button"
          variant="outline"
          size="icon"
          title="Settings"
          aria-label="Open Settings"
          onclick={() => (settingsOpen = true)}
        >
          <Settings class="h-4 w-4" />
        </Button>
      </div>
    </div>
  </header>

  <!-- Main Workspace -->
  <main class="mx-auto w-full max-w-4xl flex-1 px-5 py-5 space-y-4">
    <!-- Top Bar: Bank Name Input & Editable Output Folder -->
    <section
      class="rounded-xl border border-zinc-800/85 bg-zinc-900/50 p-4 shadow-xs"
    >
      <div class="flex flex-col sm:flex-row sm:items-end gap-3">
        <!-- Bank Name Input -->
        <div class="sm:w-64 space-y-1">
          <label
            for="bank-name-input"
            class="block text-xs font-medium text-zinc-400"
          >
            Bank Name
          </label>
          <div class="relative flex items-center">
            <Input
              id="bank-name-input"
              bind:value={bankName}
              placeholder="e.g. Weapons, UI"
              class="pr-14 font-mono text-sm h-9 bg-zinc-950/80 border-zinc-800"
            />
            <span
              class="pointer-events-none absolute right-3 text-xs font-mono text-amber-400/80"
            >
              .bank
            </span>
          </div>
        </div>

        <!-- Editable Output Folder with Direct Native Folder Picker on "Change" -->
        <div class="flex-1 space-y-1">
          <div class="flex items-center justify-between text-xs text-zinc-400">
            <label for="output-folder-input">Output Folder</label>
            <button
              type="button"
              onclick={pickOutputFolder}
              class="inline-flex items-center gap-1 text-[11px] font-medium text-amber-400 hover:text-amber-300 underline cursor-pointer"
            >
              <FolderOpen class="h-3 w-3" />
              Change
            </button>
          </div>
          <Input
            id="output-folder-input"
            bind:value={outputDirInput}
            onblur={() => persistOutputDirectory(outputDirInput)}
            placeholder="./BakedBanks"
            class="h-9 font-mono text-xs bg-zinc-950/80 border-zinc-800"
          />
        </div>
      </div>
    </section>

    <!-- Large Drag-and-Drop Audio Import Area -->
    <section
      aria-label="Audio import dropzone"
      ondragover={(e) => {
        e.preventDefault();
        isDragging = true;
      }}
      ondragleave={() => {
        isDragging = false;
      }}
      ondrop={(e) => {
        e.preventDefault();
        isDragging = false;
      }}
      class="relative rounded-2xl border-2 border-dashed transition-all duration-150 {isDragging
        ? 'border-amber-400 bg-amber-500/10 shadow-lg shadow-amber-500/5'
        : events.length === 0
          ? 'border-zinc-800 hover:border-amber-500/50 bg-zinc-900/35 py-9 px-6'
          : 'border-zinc-800/80 hover:border-amber-500/45 bg-zinc-900/25 py-4 px-5'}"
    >
      {#if events.length === 0}
        <div class="flex flex-col items-center justify-center text-center space-y-3">
          <div
            class="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/10 border border-amber-500/25 text-amber-400"
          >
            <UploadCloud class="h-6 w-6" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-medium text-zinc-200">
              Drag & drop audio files here to create FMOD events
            </p>
            <p class="text-xs text-zinc-500">
              Supports WAV, OGG, MP3, FLAC, AIFF and WMA • Auto-assigns <code class="text-zinc-400">event:/sfx/</code> or <code class="text-zinc-400">event:/music/</code> paths
            </p>
          </div>
          <div class="flex flex-wrap items-center justify-center gap-2.5 pt-1">
            <Button
              type="button"
              variant="default"
              size="sm"
              class="h-8 px-3.5"
              onclick={browseAudioFiles}
            >
              <FolderPlus class="h-3.5 w-3.5" />
              Select Audio Files
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="h-8 px-3"
              onclick={loadBundledSampleFiles}
            >
              <Sparkles class="h-3.5 w-3.5 text-amber-400" />
              Load FMOD Sample Audio
            </Button>
          </div>
        </div>
      {:else}
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2.5">
            <div
              class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-500/10 border border-amber-500/25 text-amber-400"
            >
              <UploadCloud class="h-4 w-4" />
            </div>
            <div>
              <div class="text-xs font-medium text-zinc-200">
                Drop more audio files anywhere in the window
              </div>
              <div class="text-[11px] text-zinc-500">
                WAV, OGG, MP3, FLAC, AIFF, WMA
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onclick={browseAudioFiles}
            >
              <FolderPlus class="h-3.5 w-3.5" />
              Add Files
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onclick={clearAllEvents}
              class="text-zinc-400 hover:text-red-400"
            >
              <Trash2 class="h-3.5 w-3.5" />
              Clear All
            </Button>
          </div>
        </div>
      {/if}
    </section>

    <!-- Event Cards List -->
    {#if events.length > 0}
      <section class="space-y-2.5">
        <div class="flex items-center justify-between px-1">
          <div class="flex items-center gap-2">
            <h2 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
              Events ({events.length})
            </h2>
            <Badge variant="outline">
              {events.filter((e) => e.category === "SFX").length} SFX •
              {events.filter((e) => e.category === "MUSIC").length} Music
            </Badge>
          </div>
          <span class="text-[11px] text-zinc-500">
            Supports nested paths like <code class="text-zinc-400">event:/sfx/player/jump</code>
          </span>
        </div>

        <div class="space-y-2">
          {#each events as item (item.id)}
            <EventCard
              {item}
              onRemove={removeEvent}
              onUpdate={updateEvent}
            />
          {/each}
        </div>
      </section>
    {/if}

    <!-- Build Status Banner (Success / Error / Details) -->
    {#if buildStatus === "error" && buildError}
      <section
        class="rounded-xl border border-red-500/35 bg-red-500/10 p-3.5 text-xs text-red-200 flex items-start gap-2.5"
      >
        <AlertCircle class="h-4 w-4 shrink-0 text-red-400 mt-0.5" />
        <div class="min-w-0 flex-1 space-y-1">
          <div class="font-semibold text-red-300">Bank Compilation Error</div>
          <pre class="whitespace-pre-wrap font-mono text-[11px] text-red-200/90 overflow-x-auto">{buildError}</pre>
        </div>
      </section>
    {/if}

    {#if buildStatus === "success" && lastBakeResult}
      <section
        class="rounded-xl border border-emerald-500/35 bg-emerald-500/10 p-4 text-xs space-y-3"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="flex items-center gap-2.5">
            <CheckCircle2 class="h-4 w-4 shrink-0 text-emerald-400" />
            <div>
              <div class="font-semibold text-emerald-200">
                {lastBakeResult.message}
              </div>
              <div class="text-[11px] font-mono text-emerald-300/80">
                Output: {lastBakeResult.outputDirectory}
              </div>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="border-emerald-500/35 bg-emerald-950/50 text-emerald-200 hover:bg-emerald-900/50"
              onclick={openOutputFolder}
            >
              <FolderOpen class="h-3.5 w-3.5" />
              Open Output Folder
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="text-emerald-300 hover:bg-emerald-500/15"
              onclick={() => (showBuildDetails = !showBuildDetails)}
            >
              <FileCode2 class="h-3.5 w-3.5" />
              {showBuildDetails ? "Hide Details" : "GUIDs & Logs"}
              {#if showBuildDetails}
                <ChevronUp class="h-3.5 w-3.5" />
              {:else}
                <ChevronDown class="h-3.5 w-3.5" />
              {/if}
            </Button>
          </div>
        </div>

        <!-- Compiled Bank File Pills -->
        <div class="flex flex-wrap gap-2">
          {#each lastBakeResult.builtFiles as bf}
            <div
              class="inline-flex items-center gap-1.5 rounded-lg border border-emerald-500/25 bg-zinc-950/70 px-2.5 py-1 text-[11px] font-mono text-zinc-200"
            >
              <Disc3 class="h-3 w-3 text-amber-400" />
              <span>{bf.filename}</span>
              <span class="text-zinc-500">({formatBytes(bf.sizeBytes)})</span>
            </div>
          {/each}
        </div>

        {#if showBuildDetails}
          <div class="grid grid-cols-1 {lastBakeResult.guidsContent ? 'md:grid-cols-2' : ''} gap-3 pt-2 border-t border-emerald-500/20">
            {#if lastBakeResult.guidsContent}
              <div class="space-y-1">
                <div class="text-[11px] font-semibold text-emerald-300">
                  Exported GUIDs.txt
                </div>
                <pre
                  class="max-h-40 overflow-auto rounded-lg border border-zinc-800 bg-zinc-950/90 p-2.5 font-mono text-[11px] text-zinc-300"
                >{lastBakeResult.guidsContent}</pre>
              </div>
            {/if}
            <div class="space-y-1">
              <div class="text-[11px] font-semibold text-emerald-300">
                FMOD Compiler Output
              </div>
              <pre
                class="max-h-40 overflow-auto rounded-lg border border-zinc-800 bg-zinc-950/90 p-2.5 font-mono text-[11px] text-zinc-400"
              >{lastBakeResult.compilerStdout}</pre>
            </div>
          </div>
        {/if}
      </section>
    {/if}
  </main>

  <!-- Sticky Footer Action Bar with Prominent "Bake Bank" Button & Build Status Indicator -->
  <footer
    class="sticky bottom-0 z-30 border-t border-zinc-800/85 bg-zinc-950/90 backdrop-blur-md"
  >
    <div
      class="mx-auto flex max-w-4xl items-center justify-between gap-4 px-5 py-3.5"
    >
      <!-- Build Status Indicator -->
      <div class="flex items-center gap-2.5 text-xs">
        {#if buildStatus === "baking"}
          <Loader2 class="h-4 w-4 animate-spin text-amber-400" />
          <span class="text-amber-300 font-medium">
            Baking {bankName.trim() || "SoundBank"}.bank via FMOD Studio CLI...
          </span>
        {:else if buildStatus === "success"}
          <span class="h-2 w-2 rounded-full bg-emerald-400"></span>
          <span class="text-zinc-300">
            Build Ready • Last baked <strong class="text-emerald-300">{lastBakeResult?.bankName}.bank</strong>
          </span>
        {:else if buildStatus === "error"}
          <span class="h-2 w-2 rounded-full bg-red-400"></span>
          <span class="text-red-300">Build Failed — check error above</span>
        {:else}
          <span
            class="h-2 w-2 rounded-full {events.length > 0 && compilerInfo.available
              ? 'bg-amber-400'
              : 'bg-zinc-600'}"
          ></span>
          <span class="text-zinc-400">
            {#if events.length === 0}
              Ready • Drop audio files to begin
            {:else}
              Ready to bake <strong class="text-zinc-200">{events.length}</strong>
              event{events.length === 1 ? "" : "s"} into
              <strong class="text-amber-300 font-mono">{bankName.trim() || "SoundBank"}.bank</strong>
            {/if}
          </span>
        {/if}
      </div>

      <!-- Prominent "Bake Bank" Button -->
      <Button
        type="button"
        variant="default"
        size="lg"
        disabled={buildStatus === "baking" ||
          events.length === 0 ||
          !compilerInfo.available ||
          !bankName.trim()}
        onclick={handleBakeBank}
        class="min-w-[165px] font-semibold tracking-wide shadow-md shadow-amber-500/15"
      >
        {#if buildStatus === "baking"}
          <Loader2 class="h-4 w-4 animate-spin" />
          Baking Bank...
        {:else}
          <Flame class="h-4 w-4 fill-zinc-950/20" />
          Bake Bank
        {/if}
      </Button>
    </div>
  </footer>

  <!-- Settings Modal -->
  <SettingsModal
    openModal={settingsOpen}
    {config}
    {compilerInfo}
    {isDebugBuild}
    onClose={() => (settingsOpen = false)}
    onSave={handleSaveConfig}
    onRecheckCompiler={refreshCompilerStatus}
  />
</div>
