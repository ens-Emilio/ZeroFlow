<script lang="ts">
  // Overlay window shell: screenshot backdrop + lasso canvas + toolbar.
  // Phase 0: validates transparent/clickable overlay + lasso drawing.
  import LassoOverlay from "$lib/overlay/LassoOverlay.svelte";
  import { overlaySession, toast, showToast } from "$lib/stores/index.ts";
  import { backend, events } from "$lib/ipc.ts";
  import { onMount } from "svelte";

  let backdrop: HTMLImageElement | undefined = $state();

  onMount(async () => {
    const unlisten = await events.onHotkey(async () => {
      // Hotkey pressed while overlay is open: close it.
      await backend.hideOverlay();
    });
    return unlisten;
  });

  async function onComplete(
    bounds: { x: number; y: number; width: number; height: number },
    _path: { x: number; y: number }[]
  ) {
    overlaySession.selectionBounds = bounds;
    // Phase 0: stub — grab frame + crop is not implemented on the Rust side yet.
    showToast("Selection captured — providers arrive in Phase 1");
  }

  function onCancel() {
    showToast("Selection cancelled");
    void backend.hideOverlay();
  }
</script>

{#if overlaySession.screenshotDataUrl}
  <img class="backdrop" src={overlaySession.screenshotDataUrl} alt="" bind:this={backdrop} />
{/if}

<LassoOverlay {onComplete} {onCancel} />

{#if toast.visible}
  <div class="toast" class:error={toast.isError}>{toast.message}</div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    user-select: none;
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
