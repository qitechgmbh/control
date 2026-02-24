# Electron → SolidJS Migration Analysis

## Why SolidJS is a Good Fit

The electron frontend has two fundamental issues that SolidJS directly solves:

1. **React's reconciler triggers re-renders on any state change** — the codebase added an entire `ThrottledStoreUpdater` class (100+ lines) to cap re-renders at 30 FPS and prevent UI jank from high-frequency socket events.
   In SolidJS, reactive updates are surgical: only the DOM nodes that actually depend on a changed signal update. No reconciliation pass, no virtual DOM diff. The `batch()` primitive coalesces multiple signal writes into one reactive pass — replaces `ThrottledStoreUpdater` entirely with zero boilerplate.

2. **React's hook rules and closure gotchas** force state management up into libraries (Zustand) and create a deep abstraction stack. SolidJS primitives (`createSignal`, `createStore`, `onCleanup`) live exactly where the components live and have no closure staleness issues.

---

## Abstraction Comparison: WebSocket Layer

### Electron (React)
Adding a machine namespace requires touching **5+ files**:

| File | Lines | Purpose |
|------|-------|---------|
| `socketioStore.ts` | 515 | Base infrastructure: ThrottledStoreUpdater, createNamespaceHookImplementation, namespace lifecycle |
| `{machine}Namespace.ts` | ~77 | Zod schemas, Zustand store, message handler, hook wrapper |
| `use{Machine}.ts` | ~95 | Route param parsing, optimistic state, mutation helpers |
| `routes.tsx` | +~15 | Route definitions |
| `properties.ts` | +~20 | Machine properties entry |

**Total per machine: ~220+ lines of mandatory boilerplate before writing any UI.**

### SolidJS (this PoC)
Adding a machine namespace requires:

```ts
// namespaces/myMachine.ts — ~25 lines total
export function createMyMachineNamespace(vendor, machine, serial) {
  return createNamespace(
    machineNamespacePath(vendor, machine, serial),
    (event, set) => {
      if (event.name === "StateEvent") set("data", event.data);
    },
    { data: null },
  );
}
```

Then in the page component:
```tsx
const [state] = createMyMachineNamespace(VENDOR, MACHINE, serial());
// state().data is now reactive — that's it
```

**The socket connects on component mount, disconnects on unmount, updates are reactive.
No store, no hook factory, no ThrottledStoreUpdater, no ref-counting.**

---

## The Core Primitive (`src/lib/socketio.ts`)

```
createNamespace(path, handler, initialState) → [signal, socket]
```

- `batch()` replaces `ThrottledStoreUpdater` — SolidJS batches reactive updates automatically
- `onCleanup()` replaces manual `useEffect` teardown — socket disconnects when scope disposed
- The signal IS the store — no Zustand, no Immer needed for simple machine state

This single 80-line file replaces `socketioStore.ts` (515 lines) for new machines.

---

## State Management

### Electron
- Zustand (global singletons + per-namespace stores)
- Immer (immutable updates)
- `useSyncExternalStore` (bridge Zustand → React)
- `ThrottledStoreUpdater` (performance workaround)
- `useStateOptimistic` (dual optimistic/real state)

### SolidJS
- `createSignal` / `createStore` — SolidJS built-ins
- `batch()` — built-in, no library
- Optimistic state: just another `createSignal` in the component

The optimistic pattern from `useStateOptimistic.tsx` (~80 lines) collapses to two lines in a SolidJS component:
```ts
const [optimistic, setOptimistic] = createSignal<State | null>(null);
const effective = () => optimistic() ?? serverState().data;
```

---

## Things That Don't Exist in Electron That SolidJS Gets for Free

| Feature | Electron workaround | SolidJS |
|---------|-------------------|---------|
| Fine-grained reactivity | `ThrottledStoreUpdater` (100 LOC) | `batch()` built-in |
| Cleanup on unmount | `useEffect` return + ref counting | `onCleanup()` |
| Derived state | `useMemo` | `createMemo()` or just `() => expr` |
| Conditional render | `&&` / ternary (JSX pitfalls) | `<Show>` (handles null/undefined safely) |
| List render | `.map()` | `<For>` (keyed, no full re-render on list change) |

---

## What the Full Migration Looks Like

### Things to keep / port directly
- The Zod event schemas (optional — validation is still useful)
- The REST API mutation pattern (`mutateMachine()`)
- The uplot chart components (framework-agnostic, just need a SolidJS wrapper)
- The unit/value display logic

### Things to discard
- `ThrottledStoreUpdater` — not needed
- `createNamespaceHookImplementation` — replaced by `createNamespace()`
- Zustand + Immer — replaced by SolidJS primitives
- `useStateOptimistic` — two lines in a component
- `useSyncExternalStore` bridge — not needed
- All the `useEffect` lifecycle boilerplate for socket management

### Things to re-evaluate
- **Electron itself** — this PoC runs as a plain browser app (`npm run dev`), served from the Rust backend. No Electron needed if the app runs in the browser. Consider just serving the built SolidJS app as static files from the Axum server.
- **i18next** — if the app is only used in one language, skip it
- **TanStack Router** → `@solidjs/router` handles the use case with less ceremony
- **react-hook-form + Zod** → SolidJS has no equivalent but the forms here are simple enough for bare `createSignal` + validation on submit

---

## Migration Scope Estimate

| Area | Effort | Notes |
|------|--------|-------|
| WebSocket infrastructure | Done (80 lines) | `src/lib/socketio.ts` |
| Main namespace | Done (~55 lines) | `src/namespaces/main.ts` |
| Per-machine namespace | ~25 lines each | See testMachine.ts as template |
| Machine pages (control/settings) | Medium | Mostly UI translation, logic simplifies |
| Graph components (uplot) | Medium | uplot is framework-agnostic; wrap in `onMount`/`onCleanup` |
| Sidebar + routing | Done | See App.tsx + Sidebar.tsx |
| Setup/EtherCAT pages | Small | Just display logic, no complex state |
| Presets | Medium | Local storage + fetch, straightforward |
| Electron IPC (update, theme, NixOS) | Low if dropped | Not needed if switching to browser app |

**Rough estimate: 2–3 weeks for a complete working port, 1 week for a fully functional PoC covering all machine types.**

The biggest time investment is translating the 15+ machine-specific UI pages, not the infrastructure — the infrastructure is already done and is dramatically simpler.

---

## Proof of Concept Structure

```
src/
├── lib/
│   ├── socketio.ts          # Core: createNamespace() primitive (80 lines)
│   └── api.ts               # REST mutations
├── namespaces/
│   ├── main.ts              # Main namespace (~55 lines, vs ~170 in electron)
│   ├── testMachine.ts       # Test machine namespace (~25 lines, vs ~77 in electron)
│   └── mock1.ts             # Mock machine namespace (~60 lines incl. types)
├── pages/
│   ├── MachinesPage.tsx     # Machine list
│   ├── TestMachineControlPage.tsx  # LED control with optimistic updates
│   └── Mock1ControlPage.tsx # Mock machine: live values + freq/mode controls
├── components/
│   └── Sidebar.tsx          # Navigation sidebar
└── App.tsx                  # Router + context provider
```

Run with: `npm run dev` (vite dev server, proxies to backend at localhost:3001)

---

## How Other Machines Would Be Implemented

Each machine follows the same two-step pattern:

### Step 1: Namespace file (`src/namespaces/{machine}.ts`)

Define a state type that mirrors the Rust `StateEvent` and `LiveValuesEvent` structs, then call `createNamespace()`:

```ts
export type MyMachineState = {
  // fields from StateEvent
  some_field: number | null;
  mode: "Standby" | "Running" | null;
  // fields from LiveValuesEvent (high-frequency, set separately)
  live_measurement: number | null;
};

export function createMyMachineNamespace(serial: number) {
  return createNamespace<MyMachineState>(
    machineNamespacePath(VENDOR, MACHINE_ID, serial),
    (event, set) => {
      if (event.name === "StateEvent") {
        set("some_field", event.data.some_field);
        set("mode", event.data.mode);
      } else if (event.name === "LiveValuesEvent") {
        set("live_measurement", event.data.measurement);
      }
    },
    { some_field: null, mode: null, live_measurement: null },
  );
}
```

Machine IDs (from `machines/src/lib.rs`):
| Machine | Vendor | Machine ID |
|---------|--------|-----------|
| TestMachine | 1 | 0x0033 (51) |
| Winder v2 | 1 | 0x0002 (2) |
| Extruder v2 | 1 | 0x0004 (4) |
| Laser v1 | 1 | 0x0006 (6) |
| Mock | 1 | 0x0007 (7) |
| Buffer v1 | 1 | 0x0008 (8) |
| Aquapath v1 | 1 | 0x0009 (9) |
| Wago Power v1 | 1 | 0x000A (10) |
| Extruder v3 | 1 | 0x0016 (22) |

### Step 2: Page component (`src/pages/{Machine}ControlPage.tsx`)

```tsx
export default function MyMachineControlPage() {
  const params = useParams<{ serial: string }>();
  const serial = () => parseInt(params.serial);
  const [state] = createMyMachineNamespace(serial());

  // Optimistic state per mutable field
  const [optimisticMode, setOptimisticMode] = createSignal<Mode | null>(null);
  const mode = () => optimisticMode() ?? state().mode ?? "Standby";

  async function setMode(newMode: Mode) {
    setOptimisticMode(newMode);
    try {
      await mutateMachine(
        { machine_identification: { vendor: VENDOR, machine: MACHINE_ID }, serial: serial() },
        { SetMode: newMode },
      );
    } catch { setOptimisticMode(null); }
  }

  return (/* JSX */);
}
```

### Step 3: Register route + sidebar entry

In `App.tsx`:
```tsx
<Route path="/machines/mymachine/:serial/control" component={MyMachineControlPage} />
<Route path="/machines/mymachine/:serial" component={...} />
```

In `Sidebar.tsx` and `MachinesPage.tsx`, add to the slug/label maps:
```ts
"1_X": "mymachine"  // vendor_machine → slug
```

### Pages needed per machine (from electron reference)

| Machine | Pages in electron | Notes |
|---------|------------------|-------|
| TestMachine | Control | LED toggle (done) |
| Mock | Control | Frequencies + mode + live values (done) |
| Winder v2 | Control, Settings, Graphs, Presets, Manual | Complex: spool/traverse/puller motors |
| Extruder v2/v3 | Control, Settings, Graphs, Presets, Manual | Heating zones, inverter speed |
| Laser v1 | Control, Graphs, Presets, Manual | Diameter measurement, tolerance bands |
| Buffer v1 | Control, Settings | Motor control |
| Aquapath v1 | Control, Settings, Graphs | Temperature + analog output |
| Wago Power v1 | Control | Power channel display |

### Shared UI components still to build

The electron frontend has these reusable control components that would benefit from SolidJS ports:

- **`ControlCard`** — titled card wrapper (partially done with `.control-card` CSS)
- **`EditValue`** — numeric input with min/max/step/unit/default display
- **`SelectionGroup`** — segmented button toggle (done with `.mode-selector` pattern)
- **`TimeSeriesValueNumeric`** — displays latest value from a live stream
- **Graph wrapper** — uplot integration in SolidJS using `onMount`/`onCleanup`
- **`Topbar`** — tab navigation within a machine (Control / Graph / Manual / Presets)
- **`Page`** — page container with consistent padding/width
- **Preset system** — load/save named parameter sets via REST API

---

## Machine-Specific Complexity Notes

### Winder v2
Most complex machine. Has three motors (spool, traverse, puller), a tension arm, and a diameter visualisation component. The electron page (`Winder2ControlPage.tsx`) uses custom SVG components (`Spool.tsx`, `TraverseBar.tsx`, `TensionArm.tsx`). These are pure rendering components that can be ported as-is to SolidJS.

### Extruder v2 / v3
Has heating zones (up to 8 zones, each with temperature setpoint + PID state). The `HeatingZone.tsx` component in electron is shared between extruder2 and extruder3. The graph page plots temperature history via uplot.

### Laser v1
Has `DiameterVisualisation.tsx` — a custom SVG component showing the wire cross-section with tolerance bands. The control page shows live diameter + tolerance settings.

### Mock Machine
Simplest "real" machine with meaningful state: three sine wave generators with frequency controls and a running/standby mode. Ideal for testing the frontend with the mock backend (`cargo run --features mock-machine`).

