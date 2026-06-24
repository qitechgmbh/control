# Frontend

The frontend is a desktop application, an Electron shell around a React UI (`electron/`). Operators use it to run machines: every connected machine gets its own screen with live values and controls. It is built with React, [Vite](https://vite.dev/), [Tailwind](https://tailwindcss.com/), and [Radix](https://www.radix-ui.com/), with routing handled by [TanStack Router](https://tanstack.com/router).

## The client layer

`src/client/` owns the connection to the backend. `socketioStore.ts` is a [Zustand](https://zustand-demo.pmnd.rs/) store built on a Socket.IO client (`socket.io-client`) that uses a MessagePack parser for compact live data. Incoming events are validated with [Zod](https://zod.dev/) before they reach the UI, and store updates are throttled to about 30&nbsp;FPS, a buffer collects events and flushes roughly every 33&nbsp;ms, so high-rate live data never floods React with re-renders.

A main namespace (`mainNamespace.ts`) carries the list of connected machines; `useMachines` and `useClient` expose it to the UI.

## Per-machine modules

Every backend machine has a matching frontend module under `src/machines/`. Its `*Namespace.ts` re-declares the backend's events as Zod schemas that mirror the Rust `StateEvent` struct, maps incoming event names to store updates, and exposes a namespace hook (for example `useMyMachine`).

The live-data namespace is keyed by the machine identification, `/machine/{vendor}/{machine}/{serial}`, the same per-machine namespace the backend emits into, so client and server agree on where each machine's events go.

Commands travel the other way: a mutation hook (`useMachineMutate`) sends JSON to the backend's `api_mutate` handler, with optional optimistic local updates so the UI feels immediate.

## UI building blocks

Shared components live in `src/components/`, page scaffolding (`Page`, `SidebarLayout`, `Topbar`), tables, toasts, and more. Graphs (`src/components/graph/`, built on [uPlot](https://github.com/leeoniya/uPlot)) turn the streamed live values into time-series charts. Presets (`src/components/preset/` and `src/lib/preset/`) let an operator save a machine's configuration and reload it later.

## Adding a screen for a new machine

Start from the boilerplate frontend module (`src/machines/minimal_machines/BOILERPLATE_TEMPLATE/`): copy it, mirror your machine's `StateEvent` fields as Zod schemas in the `*Namespace.ts`, and wire up the page. See **Extending**.

## Build & test

The app is built with Vite, `vite-plugin-electron` produces the Electron main process at `.vite/build/main.js`. Tests run with [Vitest](https://vitest.dev/) (unit) and [Playwright](https://playwright.dev/) (end-to-end).
