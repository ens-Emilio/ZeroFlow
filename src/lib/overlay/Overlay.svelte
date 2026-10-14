<script lang="ts">
  import LassoOverlay from "$lib/overlay/LassoOverlay.svelte";
  import { overlaySession, settings, toast, showToast } from "$lib/stores/index.svelte";
  import { backend, events } from "$lib/ipc.ts";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  // Overlay is born visible (maximized) by config; we hide it with CSS.
  let isVisible = $state(false);
  // "lasso" = drawing; "crop" = focused crop shown with the action toolbar.
  let mode = $state<"lasso" | "crop" | null>(null);

  onMount(() => {
    void backend.debugLog("Overlay mounted");
    void (async () => {
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
    })();


    // Backend shows/hides the real window; we just track state and grab the
    // frame for the backdrop each time the overlay is opened.
    // Backend decides the state: non-empty payload = show with this frame,
    // empty = hide. No blind toggling (window state and UI state drifted).
    const unlistenToggle = events.onOverlayToggle((dataUrl) => {
      void backend.debugLog(`toggle-overlay len=${dataUrl?.length ?? 0}`);
      isVisible = !!dataUrl;
      if (dataUrl) {
        // Fresh lasso session: a new frame means a new capture round, even if
        // this page instance survived (HMR) with stale crop state.
        mode = null;
        overlaySession.croppedDataUrl = null;
        overlaySession.selectionBounds = null;
        overlaySession.screenshotDataUrl = dataUrl;
      } else {
        mode = null;
        overlaySession.croppedDataUrl = null;
        overlaySession.selectionBounds = null;
      }
    });

    // Legacy events (kept for compatibility with crop etc.)
    const unlistenFrame = events.onOverlayFrame((dataUrl) => {
      overlaySession.screenshotDataUrl = dataUrl;
    });
    const unlistenHotkey = events.onHotkey(async () => {
      void backend.closeOverlay();
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") void backend.closeOverlay();
    });

    return () => {
      void unlistenToggle.then((f) => f());
      void unlistenFrame.then((f) => f());
      void unlistenHotkey.then((f) => f());
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
    } catch (e) {
      void backend.debugLog(`OCR error: ${String(e)}`);
      showToast(`OCR failed: ${e}`, true);
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
          <button class="chip primary" onclick={onSearchTraceMoe}>Search anime</button>
          <button class="chip" onclick={onOcrText}>Text</button>
          <button class="chip" onclick={onScanQr}>QR</button>
          <button class="chip" onclick={onCopyImage}>Copy</button>
          <button class="chip" onclick={onSaveImage}>Save</button>
          <button class="chip" onclick={onCloseCrop}>Close</button>
        </div>
      </div>
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