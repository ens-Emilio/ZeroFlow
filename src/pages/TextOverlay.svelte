<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "../lib/ipc.ts";
  import { settings } from "../lib/stores/index.svelte.ts";

  interface OcrWord {
    text: string;
    x: number;
    y: number;
    width: number;
    height: number;
  }

  interface Props {
    words: OcrWord[];
    imageDataUrl: string;
    onClose: () => void;
    onSearch: (word: string) => void;
  }

  let { words, imageDataUrl, onClose, onSearch }: Props = $props();

  let hoveredIdx = $state<number | null>(null);
  let containerRef: HTMLDivElement;

  // The crop image is shown at its natural size; we position word boxes
  // in CSS pixels (overlaySession coordinates). The image may be scaled
  // down by the preview container (max-width/max-height). We'll render
  // the words as an overlay with absolute coords matching the image.

  onMount(() => {
    // Nothing yet
  });

  function handleClick(idx: number) {
    const w = words[idx];
    if (!w) return;
    onSearch(w.text);
    onClose(); // close after search like original
  }
</script>

<div class="text-panel" bind:this={containerRef}>
  <img class="text-preview" src={imageDataUrl} alt="OCR crop" />
  <div class="words-layer">
    {#each words as w, i}
      <button
        class="word-box {hoveredIdx === i ? 'hover' : ''}"
        style="left: {w.x}px; top: {w.y}px; width: {w.width}px; height: {w.height}px;"
        onclick={() => handleClick(i)}
        onmouseenter={() => hoveredIdx = i}
        onmouseleave={() => hoveredIdx = null}
        title={w.text}
        aria-label="Search for '{w.text}'"
      >
        <span class="word-text">{w.text}</span>
      </button>
    {/each}
  </div>
  <div class="text-toolbar">
    <button class="chip" onclick={onClose}>Close</button>
  </div>
</div>

<style>
  .text-panel {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
    background: rgba(20, 20, 20, 0.45);
  }
  .text-preview {
    max-width: 70vw;
    max-height: 60vh;
    border-radius: 10px;
    box-shadow: 0 12px 48px rgba(0, 0, 0, 0.7);
  }
  .words-layer {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .words-layer .word-box {
    position: absolute;
    pointer-events: auto;
    background: rgba(124, 92, 255, 0.18);
    border: 2px solid rgba(124, 92, 255, 0.7);
    border-radius: 4px;
    padding: 2px 6px;
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s;
  }
  .words-layer .word-box:hover,
  .words-layer .word-box.hover {
    background: rgba(124, 92, 255, 0.35);
    border-color: #7c5cff;
  }
  .word-text {
    font: 13px system-ui, sans-serif;
    color: #f2f2f4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    max-width: 100%;
  }
  .text-toolbar {
    display: flex;
    gap: 10px;
    z-index: 1;
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
  .chip:hover { background: rgba(60, 60, 64, 0.9); }
</style>