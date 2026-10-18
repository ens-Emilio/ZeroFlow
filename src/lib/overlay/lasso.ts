// Port of reference/CTS/Capture/LassoPathSampler.cs + LassoBoundsCalculator.cs (geometry only).
// Samples pointer positions during a lasso drag and produces the closed path
// plus its bounding box.

export interface Point {
  x: number;
  y: number;
}

export class LassoPath {
  #points: Point[] = [];
  #minDistanceSq: number;

  constructor(minDistancePx = 2) {
    this.#minDistanceSq = minDistancePx * minDistancePx;
  }

  get points(): readonly Point[] {
    return this.#points;
  }

  get isEmpty(): boolean {
    return this.#points.length === 0;
  }

  add(p: Point): void {
    const last = this.#points[this.#points.length - 1];
    if (last) {
      const dx = p.x - last.x;
      const dy = p.y - last.y;
      // Skip jitter below the sampling distance.
      if (dx * dx + dy * dy < this.#minDistanceSq) return;
    }
    this.#points.push(p);
  }

  bounds(): { x: number; y: number; width: number; height: number } | null {
    if (this.#points.length === 0) return null;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const p of this.#points) {
      if (p.x < minX) minX = p.x;
      if (p.y < minY) minY = p.y;
      if (p.x > maxX) maxX = p.x;
      if (p.y > maxY) maxY = p.y;
    }
    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
  }

  // Point-in-polygon hit test (ray casting) — used to crop the screenshot to
  // the circled region, matching CircleFlow's crop behavior.
  contains(p: Point): boolean {
    const pts = this.#points;
    let inside = false;
    for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
      const a = pts[j]!;
      const b = pts[i]!;
      const intersects =
        a.y > p.y !== b.y > p.y &&
        p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x;
      if (intersects) inside = !inside;
    }
    return inside;
  }

  drawTo(ctx: CanvasRenderingContext2D): void {
    const pts = this.#points;
    if (pts.length < 2) return;
    ctx.beginPath();
    ctx.moveTo(pts[0]!.x, pts[0]!.y);
    for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i]!.x, pts[i]!.y);
    ctx.closePath();
    ctx.stroke();
  }
}
