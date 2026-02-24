/**
 * Mock1 graph page — rolling time-window uPlot charts.
 *
 * Two charts:
 *   1. Amplitude Sum — single series + ±0.8 threshold lines
 *   2. Individual Waves — amplitude1/2/3 overlaid
 *
 * Performance design:
 *   - UPlotGraph drives itself from a rAF loop; no reactive memos.
 *   - getSeriesData() slices only the visible window from the ring buffer on
 *     each frame rather than copying the entire buffer reactively.
 *   - The ring buffer itself is mutated in-place by the socket handler.
 */

import { createSignal } from "solid-js";
import { useParams } from "@solidjs/router";
import { createMock1SeriesBuffer } from "../namespaces/mock1";
import type { SeriesBuffer } from "../namespaces/mock1";
import UPlotGraph from "../components/UPlotGraph";
import type { HorizLine } from "../components/UPlotGraph";
import type uPlot from "uplot";

// Available time windows in seconds
const TIME_WINDOWS = [
  { label: "30 s",  value: 30 },
  { label: "2 min", value: 120 },
  { label: "5 min", value: 300 },
  { label: "30 min", value: 1800 },
] as const;

/**
 * Slice the ring buffer to only the entries within [tMax - windowSecs, tMax].
 * Returns typed arrays already in chronological order.
 * O(n) scan but n is at most BUFFER_SIZE and only called once per rAF frame.
 */
function windowedSlice(buf: SeriesBuffer, windowSecs: number): [Float64Array, Float64Array] {
  const { times, values, length, head } = buf;
  if (length === 0) return [new Float64Array(0), new Float64Array(0)];

  const bufSize = times.length; // BUFFER_SIZE
  const isFull = length === bufSize;

  // Determine chronological start index in the ring
  const startIdx = isFull ? head : 0;
  const tMax = times[(head - 1 + bufSize) % bufSize]; // last written entry
  const tMin = tMax - windowSecs;

  // Count how many entries fall inside [tMin, tMax]
  let count = 0;
  for (let i = 0; i < length; i++) {
    const t = times[(startIdx + i) % bufSize];
    if (t >= tMin) count++;
  }

  const outTimes = new Float64Array(count);
  const outValues = new Float64Array(count);
  let out = 0;
  for (let i = 0; i < length; i++) {
    const idx = (startIdx + i) % bufSize;
    const t = times[idx];
    if (t >= tMin) {
      outTimes[out] = t;
      outValues[out] = values[idx];
      out++;
    }
  }
  return [outTimes, outValues];
}

export default function Mock1GraphPage() {
  const params = useParams<{ serial: string }>();
  const serial = () => parseInt(params.serial);

  // createMock1SeriesBuffer opens its own socket; it disconnects on page unmount.
  // We hold a plain JS reference to the series object — no signal dependency here.
  // The rAF loop inside UPlotGraph reads from it directly every frame.
  const [getSeriesTick] = createMock1SeriesBuffer(serial());

  const [timeWindow, setTimeWindow] = createSignal(30); // seconds

  // These callbacks are called once per animation frame by UPlotGraph.
  // They capture `getSeriesTick` to always read from the latest ring buffers.
  function getSumData(): uPlot.AlignedData {
    const s = getSeriesTick();
    const [t, v] = windowedSlice(s.amplitude_sum, timeWindow());
    return [t, v];
  }

  function getAllWavesData(): uPlot.AlignedData {
    const s = getSeriesTick();
    const tw = timeWindow();
    const [t, v1] = windowedSlice(s.amplitude1, tw);
    const [, v2]  = windowedSlice(s.amplitude2, tw);
    const [, v3]  = windowedSlice(s.amplitude3, tw);
    return [t, v1, v2, v3];
  }

  const sumSeries: uPlot.Series[] = [
    { label: "Sum", stroke: "#6c8cff", width: 2 },
  ];

  const allWavesSeries: uPlot.Series[] = [
    { label: "Wave 1", stroke: "#6c8cff", width: 1.5 },
    { label: "Wave 2", stroke: "#f472b6", width: 1.5 },
    { label: "Wave 3", stroke: "#34d399", width: 1.5 },
  ];

  const sumLines: HorizLine[] = [
    { value:  0.8, color: "#6c8cff", dash: true, label: "threshold" },
    { value: -0.8, color: "#6c8cff", dash: true },
  ];

  return (
    <div class="page page-wide">
      <h1>Mock Machine — Graph</h1>
      <p class="serial">Serial: #{serial()}</p>

      <div class="graph-toolbar">
        <span class="graph-toolbar-label">Window</span>
        {TIME_WINDOWS.map((w) => (
          <button
            class={`graph-window-btn${timeWindow() === w.value ? " active" : ""}`}
            onClick={() => setTimeWindow(w.value)}
          >
            {w.label}
          </button>
        ))}
      </div>

      <div class="graph-stack">
        <div class="graph-card">
          <div class="graph-card-title">Amplitude Sum</div>
          <UPlotGraph
            getSeriesData={getSumData}
            series={sumSeries}
            unit="amp"
            height={260}
            timeWindow={timeWindow()}
            lines={sumLines}
          />
        </div>

        <div class="graph-card">
          <div class="graph-card-title">Individual Waves</div>
          <UPlotGraph
            getSeriesData={getAllWavesData}
            series={allWavesSeries}
            unit="amp"
            height={260}
            timeWindow={timeWindow()}
          />
        </div>
      </div>
    </div>
  );
}
