import type { Chart, FontSpec, Scale } from "chart.js";
import { toFont } from "chart.js/helpers";
import { HISTORY_RETENTION_MS } from "@/lib/history";
import { buildTimeTicks, formatTimeTick } from "./timeAxis";

export type ChartOptionHooks = {
  colors: () => { grid: string; axis: string } | undefined;
  renderValue: () => ((value: number) => string) | undefined;
  /** A mouse pan or wheel zoom starts on `chart`. */
  onGestureStart: (chart: Chart) => void;
  /** The mouse pan/zoom moved the x axis of `chart`; `done` at the end of the gesture. */
  onGesture: (chart: Chart, done: boolean) => void;
};

const DEFAULT_GRID = "#e2e8f0";
const DEFAULT_AXIS = "#64748b";
/** Pixels per x tick label. */
const TICK_SPACING_PX = 110;

type TimeScale = Scale & {
  qtStep?: number;
  // Chart.js internals set by `fit()`, read by the layout through `getPadding()`.
  paddingLeft: number;
  paddingRight: number;
};

/**
 * Options of a graph chart: linear time x axis with our own ticks, no animation or
 * tooltip, and mouse pan / Ctrl+wheel zoom through chartjs-plugin-zoom. Touch gestures
 * are handled by `TouchGestures`, not by the zoom plugin.
 */
export function createChartOptions(hooks: ChartOptionHooks) {
  const grid = () => hooks.colors()?.grid ?? DEFAULT_GRID;
  const axis = () => hooks.colors()?.axis ?? DEFAULT_AXIS;
  const start = ({ chart }: { chart: Chart }) => {
    hooks.onGestureStart(chart);
    return true;
  };
  const move = ({ chart }: { chart: Chart }) => hooks.onGesture(chart, false);
  const complete = ({ chart }: { chart: Chart }) =>
    hooks.onGesture(chart, true);

  return {
    animation: false as const,
    responsive: true,
    maintainAspectRatio: false,
    parsing: false as const,
    normalized: true,
    events: [
      "mousemove",
      "mouseout",
      "touchstart",
      "touchmove",
      "touchend",
      "touchcancel",
    ] as (keyof HTMLElementEventMap)[],
    interaction: { mode: "x" as const, intersect: false },
    layout: { padding: { right: 12, left: 4 } },
    scales: {
      x: {
        type: "linear" as const,
        afterBuildTicks: (scale: Scale) => {
          const maxTicks = Math.max(
            2,
            Math.floor(scale.width / TICK_SPACING_PX),
          );
          const { ticks, step } = buildTimeTicks(
            scale.min,
            scale.max,
            maxTicks,
          );
          scale.ticks = ticks;
          (scale as TimeScale).qtStep = step;
        },
        // Chart.js reserves room for half of the outermost labels, minus their distance
        // to the edge. In live mode the ticks scroll, so every tick entering or leaving
        // squeezes the plot by up to half a label. Reserve half the widest label on both
        // sides instead, whatever the tick positions. All labels of one zoom level have
        // the same format, so this only changes on zoom.
        afterFit: (scale: Scale) => {
          const s = scale as TimeScale;
          const ticks = (
            s.options as {
              ticks?: { padding?: number; font?: Partial<FontSpec> };
            }
          ).ticks;
          const ctx = s.ctx;
          ctx.save();
          ctx.font = toFont(ticks?.font ?? {}).string;
          let widest = 0;
          for (const tick of s.ticks) {
            const label = Array.isArray(tick.label)
              ? tick.label.join(" ")
              : tick.label;
            if (label)
              widest = Math.max(widest, ctx.measureText(String(label)).width);
          }
          ctx.restore();
          const pad = Math.ceil(widest / 2 + (ticks?.padding ?? 0));
          s.paddingLeft = pad;
          s.paddingRight = pad;
        },
        ticks: {
          autoSkip: false,
          maxRotation: 0,
          color: axis,
          callback: function (this: TimeScale, value: string | number) {
            return formatTimeTick(
              Number(value),
              this.qtStep ?? 1000,
              this.max - this.min,
            );
          },
        },
        grid: { color: grid },
      },
      y: {
        type: "linear" as const,
        grace: "5%",
        ticks: {
          color: axis,
          callback: (value: string | number) => {
            const render = hooks.renderValue();
            return render ? render(Number(value)) : String(value);
          },
        },
        grid: { color: grid },
      },
    },
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false },
      zoom: {
        limits: {
          // Moved along with the clock by the engine on every frame.
          x: {
            min: Date.now() - HISTORY_RETENTION_MS,
            max: Date.now(),
            minRange: 1000,
          },
        },
        pan: {
          enabled: true,
          mode: "x" as const,
          threshold: 5,
          // Mouse/pen drag only; touch is handled by TouchGestures.
          onPanStart: (ctx: {
            chart: Chart;
            event: { pointerType?: string };
          }) => (ctx.event.pointerType === "touch" ? false : start(ctx)),
          onPan: move,
          onPanComplete: complete,
        },
        zoom: {
          // Plain wheel scrolls the page; Ctrl+wheel (and trackpad pinch, which
          // Chromium reports as Ctrl+wheel) zooms.
          wheel: { enabled: true, speed: 0.1, modifierKey: "ctrl" as const },
          pinch: { enabled: false },
          mode: "x" as const,
          onZoomStart: start,
          onZoom: move,
          onZoomComplete: complete,
        },
      },
    },
  };
}
