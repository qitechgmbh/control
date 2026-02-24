/**
 * Mock1 machine namespace
 *
 * Mock machine is vendor=1, machine=7 (MACHINE_MOCK = 0x0007).
 * Events: StateEvent, LiveValuesEvent
 * Mutations: SetFrequency1/2/3(f64), SetMode("Standby"|"Running")
 */

import { createSignal, onCleanup, batch } from "solid-js";
import { io } from "socket.io-client";
import MsgPackParser from "socket.io-msgpack-parser";
import { createNamespace, machineNamespacePath, BASE_URL, genericEventSchema } from "../lib/socketio";

export type Mode = "Standby" | "Running";

export type ModeState = {
  mode: Mode;
};

export type Mock1State = {
  // From StateEvent
  is_default_state: boolean | null;
  frequency1: number | null; // millihertz
  frequency2: number | null;
  frequency3: number | null;
  mode_state: ModeState | null;
  // From LiveValuesEvent (updated at high frequency)
  amplitude_sum: number | null;
  amplitude1: number | null;
  amplitude2: number | null;
  amplitude3: number | null;
};

// Time-series buffer for graphing: parallel arrays of timestamps (s) and values
export type SeriesBuffer = {
  times: Float64Array;
  values: Float64Array;
  length: number; // number of valid entries (wraps in ring)
  head: number;   // index of next write position
};

export type Mock1Series = {
  amplitude_sum: SeriesBuffer;
  amplitude1: SeriesBuffer;
  amplitude2: SeriesBuffer;
  amplitude3: SeriesBuffer;
};

export const VENDOR_QITECH = 1;
export const MACHINE_MOCK = 7;

// Maximum number of samples retained per series (~30 min at 10 Hz = 18 000)
const BUFFER_SIZE = 20_000;

function makeBuffer(): SeriesBuffer {
  return {
    times: new Float64Array(BUFFER_SIZE),
    values: new Float64Array(BUFFER_SIZE),
    length: 0,
    head: 0,
  };
}

function pushSample(buf: SeriesBuffer, t: number, v: number): SeriesBuffer {
  buf.times[buf.head] = t;
  buf.values[buf.head] = v;
  buf.head = (buf.head + 1) % BUFFER_SIZE;
  if (buf.length < BUFFER_SIZE) buf.length++;
  return buf;
}

/** Returns sorted [times, values] arrays suitable for uPlot (oldest → newest). */
export function sortedSeries(buf: SeriesBuffer): [Float64Array, Float64Array] {
  if (buf.length === 0) return [new Float64Array(0), new Float64Array(0)];
  if (buf.length < BUFFER_SIZE) {
    // Buffer not yet full — data runs from index 0 to length-1
    return [buf.times.slice(0, buf.length), buf.values.slice(0, buf.length)];
  }
  // Ring buffer full — head points to oldest entry
  const times = new Float64Array(BUFFER_SIZE);
  const values = new Float64Array(BUFFER_SIZE);
  const tail = buf.head; // oldest index
  const firstLen = BUFFER_SIZE - tail;
  times.set(buf.times.subarray(tail), 0);
  times.set(buf.times.subarray(0, tail), firstLen);
  values.set(buf.values.subarray(tail), 0);
  values.set(buf.values.subarray(0, tail), firstLen);
  return [times, values];
}

export function createMock1Namespace(serial: number) {
  return createNamespace<Mock1State>(
    machineNamespacePath(VENDOR_QITECH, MACHINE_MOCK, serial),
    (event, set) => {
      if (event.name === "StateEvent") {
        set("is_default_state", event.data.is_default_state);
        set("frequency1", event.data.frequency1);
        set("frequency2", event.data.frequency2);
        set("frequency3", event.data.frequency3);
        set("mode_state", event.data.mode_state);
      } else if (event.name === "LiveValuesEvent") {
        set("amplitude_sum", event.data.amplitude_sum);
        set("amplitude1", event.data.amplitude1);
        set("amplitude2", event.data.amplitude2);
        set("amplitude3", event.data.amplitude3);
      }
    },
    {
      is_default_state: null,
      frequency1: null,
      frequency2: null,
      frequency3: null,
      mode_state: null,
      amplitude_sum: null,
      amplitude1: null,
      amplitude2: null,
      amplitude3: null,
    },
  );
}

/**
 * Creates a second socket connection to the same machine namespace but only
 * buffers LiveValuesEvent data into ring buffers for graphing.
 *
 * Kept separate from createMock1Namespace so the control page doesn't pay the
 * memory cost of the buffers, and the graph page manages its own connection
 * lifecycle independently.
 *
 * Returns a reactive signal: each LiveValuesEvent triggers a new object so
 * graph components can react. The buffers are mutated in-place for efficiency.
 */
export function createMock1SeriesBuffer(serial: number): [() => Mock1Series] {
  const series: Mock1Series = {
    amplitude_sum: makeBuffer(),
    amplitude1: makeBuffer(),
    amplitude2: makeBuffer(),
    amplitude3: makeBuffer(),
  };

  const [tick, setTick] = createSignal<Mock1Series>(series, { equals: false });

  const path = machineNamespacePath(VENDOR_QITECH, MACHINE_MOCK, serial);
  const socket = io(BASE_URL + path, { autoConnect: false, parser: MsgPackParser });

  socket.on("event", (raw: unknown) => {
    const parsed = genericEventSchema.safeParse(raw);
    if (!parsed.success) return;
    const event = parsed.data;
    if (event.name !== "LiveValuesEvent") return;
    const t = event.ts / 1000; // backend sends ms → convert to seconds for uPlot
    batch(() => {
      pushSample(series.amplitude_sum, t, event.data.amplitude_sum ?? 0);
      pushSample(series.amplitude1,    t, event.data.amplitude1    ?? 0);
      pushSample(series.amplitude2,    t, event.data.amplitude2    ?? 0);
      pushSample(series.amplitude3,    t, event.data.amplitude3    ?? 0);
      setTick(series);
    });
  });

  socket.connect();
  onCleanup(() => socket.disconnect());

  return [tick];
}
