// Zoom camera / loupe. Port of reference/CTS/Capture/OverlayZoomCamera.cs.
// Magnifies the area under the cursor during the drag so small text/icons are selectable.

import type { Point } from "./lasso.ts";

export interface ZoomConfig {
  scale: number;           // e.g., 2.0
  lensDiameterPx: number;  // visual size on screen
  borderWidthPx: number;
  crosshair: boolean;
}

export class OverlayZoomCamera {
  #scale: number;
  #lensRadius: number;
  #borderWidth: number;
  #crosshair: boolean;
  #ctx: CanvasRenderingContext2D | null = null;
  #sourceCanvas: HTMLCanvasElement | null = null;

  constructor(config: ZoomConfig) {
    this.#scale = config.scale;
    this.#lensRadius = config.lensDiameterPx / 2;
    this.#borderWidth = config.borderWidthPx;
    this.#crosshair = config.crosshair;
  }

  setSource(canvas: HTMLCanvasElement) {
    this.#sourceCanvas = canvas;
  }

  setContext(ctx: CanvasRenderingContext2D) {
    this.#ctx = ctx;
  }

  drawAt(center: Point): void {
    const ctx = this.#ctx;
    const src = this.#sourceCanvas;
    if (!ctx || !src || src.width === 0 || src.height === 0) return;

    const radius = this.#lensRadius;
    const scale = this.#scale;
    const srcW = radius * 2 / scale;
    const srcH = radius * 2 / scale;
    const srcX = Math.max(0, Math.min(src.width - srcW, center.x - srcW / 2));
    const srcY = Math.max(0, Math.min(src.height - srcH, center.y - srcH / 2));

    ctx.save();
    ctx.beginPath();
    ctx.arc(center.x, center.y, radius, 0, Math.PI * 2);
    ctx.clip();

    ctx.drawImage(src, srcX, srcY, srcW, srcH, center.x - radius, center.y - radius, radius * 2, radius * 2);

    ctx.beginPath();
    ctx.arc(center.x, center.y, radius - this.#borderWidth / 2, 0, Math.PI * 2);
    ctx.strokeStyle = "rgba(255,255,255,0.9)";
    ctx.lineWidth = this.#borderWidth;
    ctx.stroke();

    if (this.#crosshair) {
      const cs = radius * 0.25;
      ctx.strokeStyle = "rgba(255,255,255,0.8)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(center.x - cs, center.y);
      ctx.lineTo(center.x + cs, center.y);
      ctx.moveTo(center.x, center.y - cs);
      ctx.lineTo(center.x, center.y + cs);
      ctx.stroke();
    }

    ctx.restore();
  }
}