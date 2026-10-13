<script lang="ts">
  import LassoOverlay from "$lib/overlay/LassoOverlay.svelte";
  import { overlaySession, toast, showToast } from "$lib/stores/index.svelte";
  import { backend, events } from "$lib/ipc.ts";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  // Overlay is born visible (maximized) by config; we hide it with CSS.
  let isVisible = $state(false);

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
        overlaySession.screenshotDataUrl = dataUrl;
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
    // Search runs in the Rust process (detached task): this window is about
    // to be destroyed and must not carry the search.
    void backend.submitSelection(jpeg).catch((e) => console.error("[zf] submit failed", e));
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
    <LassoOverlay {onComplete} {onCancel} />
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
  }
  /* Dim layer sits above the screenshot so capture mode is visibly active. */
  .dim {
    position: fixed;
    inset: 0;
    background: rgba(20, 20, 20, 0.45);
    pointer-events: none;
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