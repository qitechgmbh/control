import { Chart, registerables } from "chart.js";
import annotationPlugin from "chartjs-plugin-annotation";
import "chartjs-adapter-date-fns";

let registered = false;

/**
 * Chart.js requires explicit registration of controllers/elements/scales/
 * plugins before any chart is constructed. Idempotent — safe to call from
 * every chart class's constructor.
 */
export function ensureChartJsRegistered(): void {
  if (registered) return;
  Chart.register(...registerables, annotationPlugin);
  registered = true;
}
