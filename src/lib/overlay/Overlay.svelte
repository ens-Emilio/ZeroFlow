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
  import type { OcrWord } from "$lib/stores/index.svelte";

  // Overlay is born visible (maximized) by config; we hide it with CSS.
  let isVisible = $state(false);
  // "lasso" = drawing; "crop" = focused crop shown with the action toolbar.
  // "text" = selectable words over the OCR crop.
  let mode = $state<"lasso" | "crop" | "text" | null>(null);
  let searchProvider = $state(settings.searchProviderId);
  let searchMenuOpen = $state(false);
  let translateMenuOpen = $state(false);
  let unlistenToggle = null;
  let selectedOcrWord = $state<string | null>(null);
  let unlistenFrame: UnlistenFn | null = null;
  let unlistenHotkey: UnlistenFn | null = null;

  onMount(async () => {
    void backend.debugLog("Overlay mounted");
    // The overlay window is born hidden; WebKitGTK may load this page only
    // on first show(), after toggle-overlay was already emitted. Recover:
    // if the window is visible now, adopt the pending frame.
    try {
      const win = getCurrentWindow();
      const visible = await win.isVisible();
      void backend.debugLog(`recovery: visible=${visible}`);
      if (visible) {
        const pending = await invoke<string | null>("get_pending_frame");
        if (pending) {
          isVisible = true;
          overlaySession.screenshotDataUrl = pending;
        }
      }
    } catch (e) {
      console.error("[zf] recovery failed", e);
    }
    // Legacy events (kept for compatibility with crop etc.)
    unlistenFrame = await listen("zeroflow://overlay-frame", (e: any) => {
      overlaySession.screenshotDataUrl = e.payload;
    });
    unlistenHotkey = await listen("zeroflow://hotkey", (e: any) => {
      void backend.closeOverlay();
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") void backend.closeOverlay();
    });
    window.addEventListener("click", (e) => {
      const target = e.target as HTMLElement;
      if (!target.closest(".search-dropdown")) {
        searchMenuOpen = false;
        translateMenuOpen = false;
      }
    });

    // Main toggle-overlay listener: the backend shows/hides the real window;
    // we just track state and grab the frame for the backdrop each time the
    // overlay is opened.
    let unlistenToggle: UnlistenFn | null = null;
    const registerToggleListener = async () => {
      if (unlistenToggle) unlistenToggle();
      unlistenToggle = await listen("zeroflow://toggle-overlay", (e: any) => {
        const dataUrl = e.payload; void backend.debugLog(`toggle-overlay len=${dataUrl?.length ?? 0}`);
        isVisible = !!dataUrl;
        if (dataUrl) {
          // Fresh lasso session: a new frame means a new capture round, even if
          // this page instance survived (HMR) with stale crop state.
          mode = null;
          overlaySession.croppedDataUrl = null;
          overlaySession.selectionBounds = null;
          overlaySession.screenshotDataUrl = e.payload;
        } else {
          mode = null;
          overlaySession.croppedDataUrl = null;
          overlaySession.selectionBounds = null;
        }
      });
    };
    await registerToggleListener();

    return () => {
      if (unlistenToggle) unlistenToggle();
      if (unlistenFrame) unlistenFrame();
      if (unlistenHotkey) unlistenHotkey();
    };
  });
  async function onComplete(
    bounds: { x: number; y: number; width: number; height: number },
    path: { x: number; y: number }[]
  ) {
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
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    try {
      const doc = await backend.ocr(jpeg, settings.ocrLanguageTag);
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
    void backend.debugLog("Search text chip clicked");
    // If a word was selected via hit-test, search it; otherwise search
    // the whole recognized text.
    const query = selectedOcrWord
      ?? (overlaySession.ocrWords ?? []).map((w) => w.text).join(" ").trim();
    if (!query) {
      showToast("No text; run Text first", true);
      return;
    }
    try {
      await backend.openSearch(query, settings.textSearchEngine);
      showToast(`Searching: ${query.slice(0, 60)}`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }

  async function onOcrTranslate() {
    void backend.debugLog("Translate chip clicked");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    const text = (overlaySession.ocrWords ?? []).map((w) => w.text).join(" ").trim();
    if (!text) {
      showToast("No text; run Text first", true);
      return;
    }
    try {
      const result = await backend.translateText(text, settings.targetLang);
      const translated = result.translatedText;
      void navigator.clipboard.writeText(translated).catch(() => {});
      showToast(`Translated: ${translated.slice(0, 60)}`);
    } catch (e) {
      showToast(`Translation failed: ${e}`, true);
    }
  }

  async function onOcrTranslateImage() {
    void backend.debugLog("Translate image chip clicked");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    try {
      const result = await backend.translateImage(jpeg, settings.targetLang);
      const translatedB64 = result.translatedPngBase64;
      void backend.debugLog(`translate image ok, ${translatedB64.length} b64 chars`);
      // Replace the crop preview with the translated image.
      overlaySession.croppedDataUrl = `data:image/png;base64,${translatedB64}`;
      showToast("Translated image ready (preview updated)");
    } catch (e) {
      showToast(`Translation failed: ${e}`, true);
    }
  }

  async function onOcrSelect() {
    void backend.debugLog("Select words chip clicked");
    if (!overlaySession.ocrWords || overlaySession.ocrWords.length === 0) {
      showToast("No OCR data; run Text first", true);
      return;
    }
    mode = "text";
  }

  function selectProvider(id: string) {
    void backend.debugLog(`select provider: ${id}`);
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
      switch (searchProvider) {
        case "tracemoe":
          await backend.searchTraceMoe(jpeg);
          break;
        case "googlelens":
          await backend.openLens(jpeg);
          break;
        case "yandeximages":
          await backend.searchYandex(jpeg);
          break;
        case "pinterest":
          await backend.searchPinterest(jpeg);
          break;
      }
      showToast(`Searching via ${searchProvider}...`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }


  async function onOcrWordSearch(word: string) {
    selectedOcrWord = word;
    void backend.debugLog(`selected OCR word: ${word}`);
    try {
      await backend.openSearch(word, settings.textSearchEngine);
      showToast(`Searching: ${word}`);
    } catch (e) {
      showToast(`Search failed: ${e}`, true);
    }
  }

  async function onScanQr() {
    void backend.debugLog("QR chip clicked");
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) {
      void backend.debugLog("QR: no croppedDataUrl");
      return;
    }
    void backend.debugLog(`QR: invoking scan_qr, ${jpeg.length} b64 chars`);
    try {
      const qr = await backend.scanQr(jpeg);
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

  async function onSearchTraceMoe() {
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    void backend.submitSelection(jpeg).catch((e) => console.error("[zf] submit failed", e));
    void backend.closeOverlay();
  }

  function onCopyImage() {
    const jpeg = overlaySession.croppedDataUrl?.split(",")[1];
    if (!jpeg) return;
    void backend.copyImage(jpeg).catch((e) => console.error("[zf] copy failed", e));
    showToast("Copied to clipboard");
  }

  async function onSaveImage() {
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
    void backend.closeOverlay();
  }

  function onCancel() {
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
        <div class="crop-toolbar">
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
      <LassoOverlay {onComplete} {onCancel} />
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
    /* Dim over the frozen frame so the capture mode is visibly active. */
    background: rgba(20, 20, 20, 0.45);
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
  }
  .crop-panel {
    position: fixed;
    inset: 0;
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
  .caret-btn {
    padding: 9px 10px;
    border-left: none;
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
    background: #7c5cff;
    border-color: #7c5cff;
  }
  .caret-btn:hover {
    background: #6a4be8;
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
</style>