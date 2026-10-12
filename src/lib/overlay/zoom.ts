// Zoom camera / loupe. Port of CTS/Capture/OverlayZoomCamera.cs.
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

  // Draw the magnified view at the given cursor position (CSS px).
  drawAt(center: Point): void {
    const ctx = this.#ctx;
    const src = this.#sourceCanvas;
    if (!ctx || !src) return;

    const dpr = window.devicePixelRatio || 1;
    const radius = this.#lensRadius;
    const scale = this.#scale;

    // Source rect in CSS px, centered on cursor.
    const srcX = center.x - radius / scale;
    const srcY = center.y - radius / scale;
    const srcW = radius * 2 / scale;
    const srcH = radius * 2 / scale;

    // Destination: circle centered at cursor.
    const dstX = center.x - radius;
    const dstY = center.y - radius;
    const dstW = radius * 2;
    const dstH = radius * 2;

    ctx.save();
    // Clip to circle
    ctx.beginPath();
    ctx.arc(center.x, center.y, radius, 0, Math.PI * 2);
    ctx.clip();

    // Draw magnified
    ctx.drawImage(
      src,
      srcX, srcY, srcW, srcH,
      dstX, dstY, dstW, dstH
    );

    // Border
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