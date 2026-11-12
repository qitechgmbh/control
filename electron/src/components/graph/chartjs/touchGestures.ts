import type { ViewRange } from "../history/HistoryLoader";

/** Horizontal movement before a one-finger swipe pans the graph. */
const PAN_START_PX = 16;
/** A swipe pans only if it is this many times more horizontal than vertical. */
const PAN_DIRECTION_RATIO = 2;
/** Change in finger distance before a two-finger gesture zooms. */
const PINCH_START_PX = 24;

export type TouchGestureCallbacks = {
  /** Plot area in canvas CSS pixels and the current x range. */
  getPlot: () => { left: number; width: number; range: ViewRange } | null;
  onStart: () => void;
  onMove: (range: ViewRange) => void;
  onEnd: (changed: boolean) => void;
};

type Pointer = { x: number; y: number };

/**
 * Touch pan and pinch for one chart, without blocking page scrolling. The canvas uses
 * `touch-action: pan-y`, so a vertical swipe scrolls the page natively (the browser
 * cancels our pointers). A gesture only changes the view once it is clearly horizontal
 * (one finger) or clearly a pinch (two fingers).
 */
export class TouchGestures {
  private pointers = new Map<number, Pointer>();
  private starts = new Map<number, Pointer>();
  private mode: "idle" | "pending" | "pan" | "pinch" = "idle";
  private startRange: ViewRange | null = null;
  private startDistance = 0;
  private changed = false;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly callbacks: TouchGestureCallbacks,
  ) {
    canvas.style.touchAction = "pan-y";
    canvas.addEventListener("pointerdown", this.onDown);
    canvas.addEventListener("pointermove", this.onMove);
    canvas.addEventListener("pointerup", this.onUp);
    canvas.addEventListener("pointercancel", this.onUp);
  }

  /** Hammer.js (mouse panning) may reset touch-action; keep ours. */
  enforceTouchAction() {
    if (this.canvas.style.touchAction !== "pan-y") {
      this.canvas.style.touchAction = "pan-y";
    }
  }

  dispose() {
    this.canvas.removeEventListener("pointerdown", this.onDown);
    this.canvas.removeEventListener("pointermove", this.onMove);
    this.canvas.removeEventListener("pointerup", this.onUp);
    this.canvas.removeEventListener("pointercancel", this.onUp);
  }

  private point(e: PointerEvent): Pointer {
    const rect = this.canvas.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  private onDown = (e: PointerEvent) => {
    if (e.pointerType !== "touch") return;
    const p = this.point(e);
    this.pointers.set(e.pointerId, p);
    this.starts.set(e.pointerId, p);

    if (this.pointers.size > 2) return;
    // A second finger restarts the gesture as a (pending) pinch from the current view.
    if (this.mode === "pan") this.finish();
    this.mode = "pending";
    this.startRange = this.callbacks.getPlot()?.range ?? null;
    for (const [id, pt] of this.pointers) this.starts.set(id, pt);
    this.startDistance = this.distance(this.starts);
  };

  private onMove = (e: PointerEvent) => {
    if (e.pointerType !== "touch" || !this.pointers.has(e.pointerId)) return;
    this.pointers.set(e.pointerId, this.point(e));
    if (this.mode === "idle" || !this.startRange) return;

    if (this.pointers.size === 1) this.handlePan(e.pointerId);
    else if (this.pointers.size === 2) this.handlePinch();
  };

  private onUp = (e: PointerEvent) => {
    if (e.pointerType !== "touch") return;
    this.pointers.delete(e.pointerId);
    this.starts.delete(e.pointerId);
    if (this.pointers.size === 0) {
      this.finish();
      this.mode = "idle";
    } else if (this.mode === "pinch" || this.mode === "pending") {
      // Pinch ended with one finger left: continue as a fresh pending pan.
      this.finish();
      this.mode = "pending";
      this.startRange = this.callbacks.getPlot()?.range ?? null;
      for (const [id, pt] of this.pointers) this.starts.set(id, pt);
    }
  };

  private handlePan(id: number) {
    const start = this.starts.get(id)!;
    const now = this.pointers.get(id)!;
    const dx = now.x - start.x;
    const dy = now.y - start.y;

    if (this.mode === "pending") {
      if (Math.abs(dx) < PAN_START_PX) return;
      if (Math.abs(dx) < Math.abs(dy) * PAN_DIRECTION_RATIO) {
        // Mostly vertical: leave it to page scrolling.
        this.mode = "idle";
        return;
      }
      this.begin("pan");
    }
    if (this.mode !== "pan") return;

    const plot = this.callbacks.getPlot();
    if (!plot || plot.width <= 0) return;
    const range = this.startRange!;
    const shift = (-dx / plot.width) * (range.max - range.min);
    this.emit({ min: range.min + shift, max: range.max + shift });
  }

  private handlePinch() {
    const distance = this.distance(this.pointers);
    if (this.mode === "pending") {
      if (Math.abs(distance - this.startDistance) < PINCH_START_PX) return;
      this.begin("pinch");
    }
    if (this.mode !== "pinch" || this.startDistance <= 0) return;

    const plot = this.callbacks.getPlot();
    if (!plot || plot.width <= 0) return;
    const range = this.startRange!;
    const span = range.max - range.min;
    const newSpan = span * (this.startDistance / Math.max(1, distance));

    // Keep the time under the fingers' start centre in place.
    const centre = this.centre(this.starts);
    const frac = Math.min(1, Math.max(0, (centre.x - plot.left) / plot.width));
    const anchor = range.min + frac * span;
    this.emit({ min: anchor - frac * newSpan, max: anchor + (1 - frac) * newSpan });
  }

  private begin(mode: "pan" | "pinch") {
    this.mode = mode;
    this.changed = false;
    this.callbacks.onStart();
  }

  private emit(range: ViewRange) {
    this.changed = true;
    this.callbacks.onMove(range);
  }

  private finish() {
    if (this.mode === "pan" || this.mode === "pinch") {
      this.callbacks.onEnd(this.changed);
    }
    this.changed = false;
  }

  private distance(points: Map<number, Pointer>): number {
    const [a, b] = [...points.values()];
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0;
  }

  private centre(points: Map<number, Pointer>): Pointer {
    const list = [...points.values()];
    const x = list.reduce((s, p) => s + p.x, 0) / Math.max(1, list.length);
    const y = list.reduce((s, p) => s + p.y, 0) / Math.max(1, list.length);
    return { x, y };
  }
}
