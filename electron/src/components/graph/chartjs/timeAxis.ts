import type { Tick } from "chart.js";

const SECOND = 1_000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

const NICE_STEPS = [
  100,
  200,
  500,
  SECOND,
  2 * SECOND,
  5 * SECOND,
  10 * SECOND,
  15 * SECOND,
  30 * SECOND,
  MINUTE,
  2 * MINUTE,
  5 * MINUTE,
  10 * MINUTE,
  15 * MINUTE,
  30 * MINUTE,
  HOUR,
  2 * HOUR,
  3 * HOUR,
  6 * HOUR,
  12 * HOUR,
  DAY,
  2 * DAY,
  7 * DAY,
];

function tzOffsetMs(ts: number): number {
  return -new Date(ts).getTimezoneOffset() * MINUTE;
}

/** Ticks on round local-time boundaries (…:00, :15, :30, midnight) for a linear ms axis. */
export function buildTimeTicks(
  min: number,
  max: number,
  maxTicks: number,
): { ticks: Tick[]; step: number } {
  const span = Math.max(1, max - min);
  const step =
    NICE_STEPS.find((s) => span / s <= Math.max(2, maxTicks)) ??
    NICE_STEPS[NICE_STEPS.length - 1];

  const offset = step >= HOUR ? tzOffsetMs(min) : 0;
  let t = Math.ceil((min + offset) / step) * step - offset;
  const ticks: Tick[] = [];
  while (t <= max && ticks.length < 1000) {
    ticks.push({ value: t });
    t += step;
  }
  return { ticks, step };
}

const pad = (n: number, len = 2) => String(n).padStart(len, "0");

export function formatTimeTick(value: number, step: number, span: number) {
  const d = new Date(value);
  const hm = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  const date = `${pad(d.getDate())}.${pad(d.getMonth() + 1)}.`;

  if (step >= DAY) return date;
  if (step < SECOND)
    return `${hm}:${pad(d.getSeconds())}.${pad(d.getMilliseconds(), 3)}`;
  if (step < MINUTE) return `${hm}:${pad(d.getSeconds())}`;
  if (span > DAY) {
    return d.getHours() === 0 && d.getMinutes() === 0 ? date : `${date} ${hm}`;
  }
  return hm;
}

/** Full timestamp for tooltips / cursor read-outs. */
export function formatTimestamp(value: number) {
  const d = new Date(value);
  return `${pad(d.getDate())}.${pad(d.getMonth() + 1)}.${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}
