<script lang="ts">
  import LassoOverlay from "$lib/overlay/LassoOverlay.svelte";
import TextOverlay from "../../pages/TextOverlay.svelte";
  import { overlaySession, settings, toast, showToast } from "$lib/stores/index.svelte";
  import { backend } from "$lib/ipc.ts";
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { playSound } from "$lib/sound.ts";

  function withTimeout<T>(p: Promise<T>, ms: number, label: string): Promise<T> {
    return Promise.race([p, new Promise<never>((_, rej) => setTimeout(() => rej(new Error(`${label} timed out (${ms/1000}s)`)), ms))]) as Promise<T>;
  }

  // Overlay is born visible (maximized) by config; we hide it with CSS.
  let isVisible = $state(false);
  // "lasso" = drawing; "crop" = focused crop shown with the action toolbar.
  // "text" = selectable words over the OCR crop.
  let mode = $state<"lasso" | "crop" | "text" | null>(null);
  let searchProvider = $state(settings.searchProviderId);
  let searchMenuOpen = $state(false);
  let translateMenuOpen = $state(false);
  let translationResult = $state<string | null>(null);
  let unlistenToggle: UnlistenFn | null = null;
  let selectedOcrWord = $state<string | null>(null);
  let unlistenFrame: UnlistenFn | null = null;
  let unlistenHotkey: UnlistenFn | null = null;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (translationResult) {
      e.stopPropagation();
      translationResult = null;
    } else {
      void backend.closeOverlay();
    }
  }

  function handleDocumentClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".search-dropdown")) {
      searchMenuOpen = false;
      translateMenuOpen = false;
    }
  }

  function handleToggle(e: any) {
    const dataUrl = e.payload;
    void backend.debugLog(`toggle-overlay len=${dataUrl?.length ?? 0}`);
    isVisible = !!dataUrl;
    mode = null;
    translationResult = null;
    selectedOcrWord = null;
    overlaySession.ocrWords = [];
    overlaySession.croppedDataUrl = null;
    overlaySession.selectionBounds = null;
    if (!dataUrl) return;
    playSound("overlay_opened");
    overlaySession.screenshotDataUrl = dataUrl;
    setTimeout(async () => {
      if (overlaySession.screenshotDataUrl) return;
      try {
        const pending = await invoke<string | null>("get_pending_frame");
        if (pending) overlaySession.screenshotDataUrl = pending;
      } catch {
        // The overlay can close while the recovery request is in flight.
      }
    }, 300);
  }

  onMount(() => {
    void (async () => {
      void backend.debugLog("Overlay mounted");
      try {
        const win = getCurrentWindow();
        const visible = await win.isVisible();
        void backend.debugLog(`recovery: visible=${visible}`);
        if (visible) {
          const pending = await invoke<string | null>("get_pending_frame");
          if (pending) {
            selectedOcrWord = null;
            overlaySession.ocrWords = [];
            overlaySession.croppedDataUrl = null;
            overlaySession.selectionBounds = null;
            translationResult = null;
            isVisible = true;
            overlaySession.screenshotDataUrl = pending;
          }
        }
      } catch (e) {
        console.error("[zf] recovery failed", e);
      }
      unlistenFrame = await listen("zeroflow://overlay-frame", (e: any) => {
        overlaySession.screenshotDataUrl = e.payload;
      });
      unlistenHotkey = await listen("zeroflow://hotkey", () => {
        void backend.closeOverlay();
      });
      window.addEventListener("keydown", handleKeydown);
      window.addEventListener("click", handleDocumentClick);
      const registerToggleListener = async () => {
        if (unlistenToggle) unlistenToggle();
        unlistenToggle = await listen("zeroflow://toggle-overlay", handleToggle);
      };
      await registerToggleListener();
    })();

    return () => {
      if (unlistenToggle) unlistenToggle();
      if (unlistenFrame) unlistenFrame();
      if (unlistenHotkey) unlistenHotkey();
      window.removeEventListener("keydown", handleKeydown);
      window.removeEventListener("click", handleDocumentClick);
    };
  });
  async function onComplete(
    bounds: { x: number; y: number; width: number; height: number },
    path: { x: number; y: number }[]
  ) {
    selectedOcrWord = null;
    overlaySession.ocrWords = [];
    translationResult = null;
    overlaySession.croppedDataUrl = null;
    overlaySession.selectionBounds = bounds;
    let jpeg: string;
    try {
      jpeg = await backend.cropSelection(path);
    } catch (e) {
      showToast(`Crop failed: ${e}`, true);
      return;
    }
    // Show the focused crop with the action toolbar; the user picks what
    // to do with it (search, copy, save) before the overlay goes away.
    overlaySession.croppedDataUrl = `data:image/jpeg;base64,${jpeg}`;
    mode = "crop";
    void backend.debugLog(`crop done, mode=crop, jpeg=${jpeg.length} b64, visible=${isVisible}`);
  }

  async function onOcrText() {
    void backend.debugLog("Text chip clicked");
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    try {
      const doc = await withTimeout(backend.ocr(jpeg, settings.ocrLanguageTag), 15_000, "OCR");
      void backend.debugLog(`OCR: ${doc.words.length} words`);
      if (doc.words.length === 0) {
        showToast("No text found");
        return;
      }
      const text = doc.words.map((w) => w.text).join(" ");
      void navigator.clipboard.writeText(text).catch(() => {});
      showToast(`Copied: ${text.slice(0, 60)}${text.length > 60 ? "…" : ""}`);
      overlaySession.ocrWords = doc.words;
    } catch (e) {
      void backend.debugLog(`OCR error: ${String(e)}`);
      showToast(`OCR failed: ${e}`, true);
    }
  }

  async function onOcrSearchText() {
    playSound("tap");
    // If a word was selected via hit-test, search it; otherwise search
    // the whole recognized text.
    const query = selectedOcrWord
      ?? (overlaySession.ocrWords ?? []).map((w) => w.text).join(" ").trim();
    if (!query) {
      showToast("No text; run Text first", true);
      return;
    }
    try {
      await withTimeout(backend.openSearch(query, searchProvider, settings.textSearchEngineId), 10_000, "Search");
      showToast(`Searching: ${query.slice(0, 60)}`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }

  async function onOcrTranslate() {
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    const text = (overlaySession.ocrWords ?? []).map((w) => w.text).join(" ").trim();
    if (!text) {
      showToast("No text; run Text first", true);
      return;
    }
    try {
      const result = await withTimeout(backend.translateText(text, settings.targetLang), 15_000, "Translate");
      const translated = result.translatedText.trim();
      if (!translated) {
        showToast("Translation returned no text", true);
        return;
      }
      translationResult = translated;
      void navigator.clipboard.writeText(translated).catch(() => {});
      showToast(`Translated and copied: ${translated.slice(0, 60)}`);
    } catch (e) {
      showToast(`Translation failed: ${e}`, true);
    }
  }

  async function onOcrTranslateImage() {
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    try {
      const result = await withTimeout(backend.translateImage(jpeg, settings.targetLang) as Promise<any>, 25_000, "Image translate");
      const translatedB64 = result.translatedPngBase64;
      void backend.debugLog(`translate image ok, ${translatedB64.length} b64 chars`);
      // Replace the crop preview with the translated image.
      overlaySession.croppedDataUrl = `data:image/png;base64,${translatedB64}`;
      showToast("Translated image ready (preview updated)");
    } catch (e) {
      showToast(`Translation failed: ${e}`, true);
    }
  }

  let musicBusy = $state(false);
  let musicUnlisten: (() => void) | null = null;
  async function stopMusic() {
    try {
      await backend.stopMusicRecognition();
      showToast("Music listening stopped");
    } catch (e) {
      showToast(`Could not stop music listening: ${e}`, true);
    }
    musicBusy = false;
    try { musicUnlisten?.(); } catch {}
    musicUnlisten = null;
  }

  async function onMusic() {
    playSound("tap");
    if (musicBusy) {
      await stopMusic();
      return;
    }
    musicBusy = true;
    try {
      void backend.debugLog("Music chip clicked");
      showToast("Listening... keep music playing (12s)");
      const { listen } = await import("@tauri-apps/api/event");
      let keepAlive: ReturnType<typeof setInterval> | null = null;
      const unlisten = await listen("zeroflow://music-result", (e: any) => {
        const r = e.payload as any;
        void backend.debugLog(`Music result: ${JSON.stringify(r)}`);
        if (keepAlive) { clearInterval(keepAlive); keepAlive = null; }
        musicBusy = false;
        try { unlisten(); musicUnlisten = null; } catch {}
        if (r.status === "stub_ok") showToast(`Capture OK rms=${(r.rms as number ?? 0).toFixed(4)}; Shazam next build`);
        else if (r.status === "matched") {
          const rec = r.recognition ?? r.track;
          showToast(`Matched: ${rec.title} - ${rec.artist}`);
          if (rec.share_url) void backend.openUrl(rec.share_url).catch(() => {});
        } else if (r.status === "no_audio") showToast("No audio captured (is music playing?)", true);
        else if (r.status === "no_match") showToast("No match; try louder / longer", true);
        else if (r.status === "rate_limited") showToast("Shazam rate-limited, retry later", true);
        else showToast((r.error as string) ?? "Music error", true);
      });
      musicUnlisten = unlisten;
      await backend.startMusicRecognition();
    } catch (e) {
      void backend.debugLog(`Music error: ${String(e)}`);
      showToast(String(e), true);
      musicBusy = false;
      try { musicUnlisten?.(); } catch {}
      musicUnlisten = null;
    }
  }

  async function onOcrSelect() {
    playSound("tap");
    if (!overlaySession.ocrWords || overlaySession.ocrWords.length === 0) {
      showToast("No OCR data; run Text first", true);
      return;
    }
    mode = "text";
  }

  function selectProvider(id: string) {
    playSound("tap");
    searchProvider = id;
    settings.searchProviderId = id;
    searchMenuOpen = false;
    void onSearchProvider();
  }

  async function onSearchProvider() {
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    void backend.debugLog(`search via ${searchProvider}, jpeg=${jpeg.length} b64`);
    try {
      await withTimeout(backend.searchVisual(searchProvider, jpeg), 25_000, `Search (${searchProvider})`);
      showToast(`Searching via ${searchProvider}...`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }


  async function onOcrWordSearch(word: string) {
    selectedOcrWord = word;
    void backend.debugLog(`selected OCR word: ${word}`);
    try {
      await withTimeout(backend.openSearch(word, searchProvider, settings.textSearchEngineId), 10_000, "Search");
      showToast(`Searching: ${word}`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }

  async function onScanQr() {
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) {
      void backend.debugLog("QR: no croppedDataUrl");
      return;
    }
    void backend.debugLog(`QR: invoking scan_qr, ${jpeg.length} b64 chars`);
    try {
      const qr = await withTimeout(backend.scanQr(jpeg) as Promise<any>, 10_000, "QR");
      void backend.debugLog(`QR: scan returned ${JSON.stringify(qr)}`);
      if (qr) {
        showToast(`QR: ${qr.text.slice(0, 60)}`);
        void backend.debugLog(`QR found: ${qr.text}`);
        if (/^https?:\/\//.test(qr.text)) {
          // Open the decoded URL in the default browser.
          void backend.openUrl(qr.text).catch((e) =>
            console.error("[zf] open url failed", e)
          );
        }
      } else {
        showToast("No QR code found");
      }
    } catch (e) {
      void backend.debugLog(`QR scan error: ${String(e)}`);
      showToast(`QR scan failed: ${e}`, true);
    }
  }


  function onCopyImage() {
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    void backend.copyImage(jpeg).catch((e) => console.error("[zf] copy failed", e));
    showToast("Copied to clipboard");
  }

  async function onSaveImage() {
    playSound("tap");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    try {
      const path = await backend.saveImage(jpeg);
      showToast(path ? `Saved to ${path}` : "Save failed", !path);
    } catch (e) {
      showToast(`Save failed: ${e}`, true);
    }
  }

  function onCloseCrop() {
    playSound("tap");
    void backend.closeOverlay();
  }

  function onCancel() {
    playSound("tap");
    void backend.closeOverlay();
  }
</script>

{#if isVisible}
  <div class="overlay-container">
    {#if overlaySession.screenshotDataUrl}
      <img class="backdrop" src={overlaySession.screenshotDataUrl} alt="" />
    {/if}
    <div class="dim"></div>
    {#if mode === "crop" && overlaySession.croppedDataUrl}
      <div class="crop-panel">
        <img class="crop-preview" src={overlaySession.croppedDataUrl} alt="Selection" />
        {#if translationResult}
          <section class="translation-result" aria-live="polite" aria-label="Translation result">
            <div class="translation-heading">
              <h2>Translation</h2>
              <button class="translation-close" type="button" aria-label="Close translation" onclick={() => (translationResult = null)}>×</button>
            </div>
            <p>{translationResult}</p>
          </section>
        {/if}
        <div class="crop-toolbar" role="toolbar" aria-label="Selection actions">
          <div class="search-dropdown">
            <button class="chip primary" onclick={() => (searchMenuOpen = !searchMenuOpen)}>
              Search
              <span class="caret">▾</span>
            </button>
            {#if searchMenuOpen}
            <div class="search-menu open">
              <button class="menu-item" onclick={() => selectProvider("tracemoe")}>
                <span class="menu-icon">🔍</span> trace.moe
              </button>
              <button class="menu-item" onclick={() => selectProvider("googlelens")}>
                <span class="menu-icon">🌐</span> Google Lens
              </button>
              <button class="menu-item" onclick={() => selectProvider("yandeximages")}>
                <span class="menu-icon">🖼️</span> Yandex Images
              </button>
              <button class="menu-item" onclick={() => selectProvider("pinterest")}>
                <span class="menu-icon">📌</span> Pinterest
              </button>
            </div>
            {/if}
          </div>
          <button class="chip" onclick={onOcrText}>Text</button>
    <div class="search-dropdown">
      <button class="chip" onclick={() => (translateMenuOpen = !translateMenuOpen)}>
        Translate <span class="caret">▾</span>
      </button>
      {#if translateMenuOpen}
      <div class="search-menu open">
        <button class="menu-item" onclick={() => { translateMenuOpen = false; void onOcrTranslate(); }}>
          <span class="menu-icon">📝</span> Text (LibreTranslate)
        </button>
        <button class="menu-item" onclick={() => { translateMenuOpen = false; void onOcrTranslateImage(); }}>
          <span class="menu-icon">🖼️</span> Image (Google)
        </button>
      </div>
      {/if}
    </div>
    <button class="chip" onclick={onOcrSearchText}>Search text</button>
    <button class="chip" onclick={onOcrSelect}>Select words</button>
          <button class="chip" onclick={onMusic}>{musicBusy ? "Stop music" : "Music"}</button>
          <button class="chip" onclick={onScanQr}>QR</button>
          <button class="chip" onclick={onCopyImage}>Copy</button>
          <button class="chip" onclick={onSaveImage}>Save</button>
          <button class="chip" onclick={onCloseCrop}>Close</button>
        </div>
      </div>
    {:else if mode === "text"}
      <TextOverlay
        words={overlaySession.ocrWords}
        imageDataUrl={overlaySession.croppedDataUrl ?? ""}
        onClose={onCloseCrop}
        onSearch={onOcrWordSearch}
      />
    {:else}
      <LassoOverlay
        {onComplete}
        {onCancel}
        zoomEnabled={settings.overlayZoom}
        sourceDataUrl={overlaySession.screenshotDataUrl ?? ""}
        minDiagonalPx={settings.lassoMinDiagonalPx}
      />
    {/if}
  </div>
{/if}

{#if toast.visible}
  <div class="toast" class:error={toast.isError}>{toast.message}</div>
{/if}

<style>
  .overlay-container {
    position: fixed;
    inset: 0;
    cursor: crosshair;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    user-select: none;
    pointer-events: none;
    z-index: 0;
  }
  /* Dim layer sits above the screenshot so capture mode is visibly active. */
  .dim {
    position: fixed;
    inset: 0;
    background: rgba(20, 20, 20, 0.45);
    pointer-events: none;
    z-index: 1;
  }
  .translation-result {
    width: min(60vw, 720px);
    max-height: 24vh;
    overflow: auto;
    padding: 14px 16px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 10px;
    background: rgba(30, 30, 32, 0.94);
    color: #f2f2f4;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
  }
  .translation-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .translation-heading h2 {
    margin: 0;
    font-size: 15px;
  }
  .translation-result p {
    margin: 8px 0 0;
    white-space: pre-wrap;
    line-height: 1.45;
  }
  .translation-close {
    border: 0;
    background: transparent;
    color: inherit;
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
  }
  .crop-panel {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
  }
  .crop-preview {
    max-width: 60vw;
    max-height: 60vh;
    border-radius: 10px;
    box-shadow: 0 12px 48px rgba(0, 0, 0, 0.7);
  }
  .crop-toolbar {
    display: flex;
    gap: 10px;
  }
  .chip {
    padding: 9px 18px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.28);
    background: rgba(30, 30, 32, 0.85);
    color: #f2f2f4;
    font-size: 14px;
    cursor: pointer;
  }
  .chip:hover {
    background: rgba(60, 60, 64, 0.9);
  }
  .chip.primary {
    background: #7c5cff;
    border-color: #7c5cff;
    color: white;
  }
  .chip.primary:hover {
    background: #6a4be8;
  }
  .search-dropdown {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .search-dropdown .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .search-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 6px;
    background: rgba(30, 30, 32, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 10px;
    padding: 6px;
    min-width: 180px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    opacity: 0;
    visibility: hidden;
    transform: translateY(-8px);
    transition: opacity 0.15s, transform 0.15s, visibility 0.15s;
    z-index: 20;
  }
  .search-menu.open {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }
  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: #f2f2f4;
    font: 14px system-ui, sans-serif;
    text-align: left;
    border-radius: 6px;
    cursor: pointer;
  }
  .menu-item:hover {
    background: rgba(60, 60, 64, 0.9);
  }
  .menu-icon { font-size: 16px; }
  .toast {
    position: fixed;
    bottom: 32px;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 18px;
    border-radius: 8px;
    background: rgba(30, 30, 30, 0.92);
    color: #eee;
    font: 14px system-ui, sans-serif;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    transition: opacity 0.2s;
  }
  .toast.error {
    background: rgba(80, 20, 20, 0.92);
  }
  :global(:focus-visible) { outline: 2px solid #4285f4; outline-offset: 2px; }
</style>