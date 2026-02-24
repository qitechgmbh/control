/**
 * SolidJS Socket.IO layer
 *
 * Key insight: SolidJS signals replace both Zustand + ThrottledStoreUpdater.
 * - createSignal() is the store
 * - batch() coalesces multiple updates into one DOM pass (no manual 30fps timer needed)
 * - onCleanup() replaces useEffect cleanup
 * - No lifecycle boilerplate per machine type
 *
 * The entire pattern collapses to: connect → on("event") → setSignal(data)
 */

import { io, Socket } from "socket.io-client";
import MsgPackParser from "socket.io-msgpack-parser";
import { createSignal, onCleanup, batch } from "solid-js";
import { z } from "zod";

export const BASE_URL = "http://localhost:3001";

// ---------------------------------------------------------------------------
// Generic event envelope (mirrors the Rust GenericEvent on the server)
// ---------------------------------------------------------------------------

export const genericEventSchema = z.object({
  name: z.string(),
  data: z.any(),
  ts: z.number().int().positive(),
});

export type GenericEvent = z.infer<typeof genericEventSchema>;

// ---------------------------------------------------------------------------
// Namespace path helpers
// ---------------------------------------------------------------------------

export type NamespacePath = string; // e.g. "/main" or "/machine/1/2/100"

export function machineNamespacePath(
  vendor: number,
  machine: number,
  serial: number,
): NamespacePath {
  return `/machine/${vendor}/${machine}/${serial}`;
}

// ---------------------------------------------------------------------------
// createNamespace — the single primitive for all WebSocket connections
//
// Returns a reactive [data, socket] tuple.  The caller provides:
//   - path: the Socket.IO namespace path
//   - handler: a function (event, setData) called on each incoming event
//
// Usage inside a SolidJS component or createRoot:
//
//   const [state, _socket] = createNamespace("/main", (event, set) => {
//     if (event.name === "MachinesEvent") set("machines", event.data);
//   }, { machines: null });
//
// The socket disconnects automatically when the reactive scope is disposed
// (e.g. component unmounts, createRoot cleanup).
// ---------------------------------------------------------------------------

export function createNamespace<S extends object>(
  path: NamespacePath,
  handler: (event: GenericEvent, set: (key: keyof S, value: S[keyof S]) => void, setAll: (updater: (prev: S) => S) => void) => void,
  initialState: S,
): [() => S, Socket] {
  const [state, setState] = createSignal<S>(initialState);

  const socket = io(BASE_URL + path, {
    autoConnect: false,
    parser: MsgPackParser,
  });

  // Helper: update a single key — uses batch internally for multi-key updates
  const set = (key: keyof S, value: S[keyof S]) => {
    setState((prev) => ({ ...prev, [key]: value }));
  };

  const setAll = (updater: (prev: S) => S) => {
    setState((prev) => updater(prev));
  };

  socket.on("connect", () => {
    console.log(`[socket.io] connected: ${path}`);
  });

  socket.on("disconnect", (reason) => {
    console.warn(`[socket.io] disconnected: ${path} — ${reason}`);
    // Reset state on disconnect so UI reflects stale/loading state
    batch(() => setState(initialState));
  });

  socket.on("event", (raw: unknown) => {
    const parsed = genericEventSchema.safeParse(raw);
    if (!parsed.success) {
      console.error("[socket.io] invalid event envelope:", parsed.error);
      return;
    }
    // batch() ensures multiple set() calls in one handler only trigger one
    // reactive update pass — replaces the ThrottledStoreUpdater entirely
    batch(() => handler(parsed.data, set, setAll));
  });

  socket.connect();

  // Automatically disconnect when the reactive scope is torn down
  onCleanup(() => {
    socket.disconnect();
    console.log(`[socket.io] cleaned up: ${path}`);
  });

  return [state, socket];
}
