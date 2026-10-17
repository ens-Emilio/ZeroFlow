<script lang="ts">
  // Lasso overlay (Canvas 2D). Pointer drag -> sampled path -> bounds on release.
  import { onMount } from "svelte";
  import { LassoPath, type Point } from "./lasso.ts";
  import { backend } from "../ipc.ts";

  let {
    onComplete,
    onCancel,
    minDiagonalPx = 12,
    sampleDistancePx = 2,
    strokeColor = "#4285f4",
    strokeWidth = 2.5,
    fillColor = "rgba(66, 133, 244, 0.12)",
  }: {
    onComplete: (bounds: { x: number; y: number; width: number; height: number }, path: Point[]) => void;
    onCancel?: () => void;
    minDiagonalPx?: number;
    sampleDistancePx?: number;
    strokeColor?: string;
    strokeWidth?: number;
    fillColor?: string;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let path = new LassoPath(sampleDistancePx);
  let dragging = $state(false);
  let dpr = 1;

  function resize() {
    if (!canvas) return;
    dpr = window.devicePixelRatio || 1;
    canvas.width = window.innerWidth * dpr;
    canvas.height = window.innerHeight * dpr;
    canvas.style.width = `${window.innerWidth}px`;
    canvas.style.height = `${window.innerHeight}px`;
    redraw();
  }

  onMount(() => {
    resize();
    window.addEventListener("resize", resize);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && dragging) {
        dragging = false;
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
    // A freshly created overlay window mounts this page before it is mapped,
    // so innerWidth/Height were 0 at resize() time: re-measure now.
    if (canvas && (canvas.width === 0 || canvas.height === 0)) resize();
    void backend.debugLog(`lasso down at ${e.clientX},${e.clientY} canvas ${canvas?.width}x${canvas?.height}`);
    dragging = true;
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
    if (!dragging) return;
    dragging = false;
    const bounds = path.bounds();
    void backend.debugLog(`lasso up, pts=${path.points.length}, bounds=${JSON.stringify(bounds)}`);
    if (bounds && Math.hypot(bounds.width, bounds.height) >= minDiagonalPx) {
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
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
></canvas>

<style>
  .lasso-canvas {
    position: fixed;
    inset: 0;
    z-index: 10;
    cursor: crosshair;
    touch-action: none;
  }
</style>
