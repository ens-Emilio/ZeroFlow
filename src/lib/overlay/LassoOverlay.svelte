<script lang="ts">
  // Lasso overlay (Canvas 2D). Pointer drag -> sampled path -> bounds on release.
  import { onMount } from "svelte";
  import { LassoPath, type Point } from "./lasso.ts";
  import { OverlayZoomCamera } from "./zoom.ts";
  import { backend } from "../ipc.ts";
  import { playSound } from "../sound.ts";

  let {
    onComplete,
    onCancel,
    minDiagonalPx = 12,
    sampleDistancePx = 2,
    strokeColor = "#4285f4",
    strokeWidth = 4,
    fillColor = "rgba(66, 133, 244, 0.12)",
    zoomEnabled = false,
    sourceDataUrl = "",
  }: {
    onComplete: (bounds: { x: number; y: number; width: number; height: number }, path: Point[]) => void;
    onCancel?: () => void;
    minDiagonalPx?: number;
    sampleDistancePx?: number;
    strokeColor?: string;
    strokeWidth?: number;
    fillColor?: string;
    zoomEnabled?: boolean;
    sourceDataUrl?: string;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();

  let path = new LassoPath(2);
  let dragging = $state(false);
  let dpr = 1;
  let sourceCanvas: HTMLCanvasElement | null = null;
  let sourceImage: HTMLImageElement | null = null;
  const zoomCamera = new OverlayZoomCamera({ scale: 2, lensDiameterPx: 156, borderWidthPx: 3, crosshair: true });

  function resize() {
    if (!canvas) return;
    dpr = window.devicePixelRatio || 1;
    canvas.width = window.innerWidth * dpr;
    canvas.height = window.innerHeight * dpr;
    canvas.style.width = `${window.innerWidth}px`;
    canvas.style.height = `${window.innerHeight}px`;
    if (sourceCanvas) {
      sourceCanvas.width = window.innerWidth;
      sourceCanvas.height = window.innerHeight;
      if (sourceImage?.complete) drawSource();
    }
    redraw();
  }

  function drawSource() {
    if (!sourceCanvas || !sourceImage) return;
    const ctx = sourceCanvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, sourceCanvas.width, sourceCanvas.height);
    ctx.drawImage(sourceImage, 0, 0, sourceCanvas.width, sourceCanvas.height);
  }

  function loadSource() {
    if (!sourceDataUrl || !sourceCanvas) return;
    sourceImage = new Image();
    sourceImage.onload = () => { drawSource(); redraw(); };
    sourceImage.src = sourceDataUrl;
  }

  $effect(() => {
    sourceDataUrl;
    loadSource();
  });

  onMount(() => {
    sourceCanvas = document.createElement("canvas");
    resize();
    loadSource();
    requestAnimationFrame(() => resize());
    window.addEventListener("resize", resize);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && dragging) {
        dragging = false;
        path = new LassoPath(sampleDistancePx);
        redraw();
        onCancel?.();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("resize", resize);
      window.removeEventListener("keydown", onKey);
    };
  });

  function redraw() {
    const c = canvas;
    if (!c) return;
    const ctx = c.getContext("2d")!;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);
    if (!dragging) return;
    const pts = path.points;
    const point = pts[pts.length - 1];
    if (zoomEnabled && zoomCamera && sourceCanvas && point) {
      ctx.save();
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      zoomCamera.setSource(sourceCanvas);
      zoomCamera.setContext(ctx);
      zoomCamera.drawAt(point);
      ctx.restore();
    }
    if (pts.length < 2) return;
    ctx.strokeStyle = strokeColor;
    ctx.lineWidth = strokeWidth + 1;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.fillStyle = fillColor;
    ctx.beginPath();
    ctx.moveTo(pts[0]!.x, pts[0]!.y);
    for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i]!.x, pts[i]!.y);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    // KWin may deliver the first pointerdown before the canvas was laid out: ensure size
    if (canvas && (canvas.width === 0 || canvas.height === 0)) resize();
    // A freshly created overlay window mounts this page before it is mapped,
    // so innerWidth/Height were 0 at resize() time: re-measure now.
    if (canvas && (canvas.width === 0 || canvas.height === 0)) resize();
    void backend.debugLog(`lasso down at ${e.clientX},${e.clientY} canvas ${canvas?.width}x${canvas?.height}`);
    dragging = true;
    playSound("tap");
    path = new LassoPath(sampleDistancePx);
    path.add({ x: e.clientX, y: e.clientY });
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    redraw();
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    path.add({ x: e.clientX, y: e.clientY });
    redraw();
  }

  function onPointerUp(e: PointerEvent) {
    dragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    // guard: single point = accidental tap / WebKit lost pointermove -> ignore, don't close
    if (path.points.length < 2) { void backend.debugLog(`lasso up, pts=${path.points.length} ignored (tap)`); return; }
    const bounds = path.bounds();
    void backend.debugLog(`lasso up, pts=${path.points.length}, bounds=${JSON.stringify(bounds)}`);
    if (bounds && Math.hypot(bounds.width, bounds.height) >= minDiagonalPx) {
      playSound("selection_done");
      onComplete(bounds, [...path.points]);
    } else {
      onCancel?.();
    }
  }
</script>

<canvas
  bind:this={canvas}
  class="lasso-canvas"
  onpointerdown={onPointerDown}
  onpointerup={onPointerUp}
  onpointermove={onPointerMove}
  onpointercancel={(e) => {
    if (!dragging) return;
    dragging = false;
    path = new LassoPath(sampleDistancePx);
    redraw();
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    onCancel?.();
  }}
></canvas>

<style>
  .lasso-canvas {
    position: fixed;
    inset: 0;
    z-index: 30;
    cursor: crosshair;
    touch-action: none;
    pointer-events: auto;
    display: block;
  }
</style>
