import {
  Chart,
  LineController,
  LineElement,
  PointElement,
  LinearScale,
} from "chart.js";
import zoomPlugin from "chartjs-plugin-zoom";

let registered = false;

/** Registers only what the graphs use, keeping the bundle small. */
export function registerChartJs() {
  if (registered) return;
  Chart.register(
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    zoomPlugin,
  );
  registered = true;
}
