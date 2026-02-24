# SolidJS Frontend — Feature Parity Roadmap

This document tracks what still needs to be implemented to reach feature parity with the
existing Electron/React frontend. Checked items are complete in this PoC.

---

## Infrastructure

- [x] Socket.IO connection primitive (`createNamespace`)
- [x] Main namespace (machine discovery, EtherCAT status)
- [x] REST mutation helper (`mutateMachine`)
- [x] Sidebar with live machine list
- [x] Basic routing (`@solidjs/router`)
- [x] Dark theme CSS variables
- [ ] Topbar / tab navigation per machine (Control / Graph / Manual / Presets)
- [ ] Toast notifications (success/error feedback on mutations)
- [ ] Global error boundary
- [ ] Vite proxy config for production build (serve from Axum)

---

## Shared UI Components

- [x] Control card wrapper (`.control-card`)
- [x] Live value display (numeric, tabular-nums)
- [x] Frequency / numeric input with reset-to-default button
- [x] Mode selector (segmented button)
- [ ] `EditValue` — generalised numeric input with unit label, min/max/step, inline validation
- [ ] `SelectionGroup` — generalised segmented button for arbitrary enum values
- [ ] `TimeSeriesValueNumeric` — last-value display from a streaming signal
- [ ] uplot graph wrapper — `onMount` creates chart, reactive signals push data, `onCleanup` destroys
- [ ] Preset system UI — list presets, apply preset, save as preset (uses REST API)
- [ ] Manual/PDF viewer page
- [ ] EtherCAT setup page (device list, address assignments)
- [ ] Setup / configuration page (EtherCAT interface picker)

---

## Machine Pages

### Test Machine (vendor=1, machine=0x33/51)
- [x] Control page — LED toggle grid

### Mock Machine (vendor=1, machine=0x07/7)
- [x] Control page — live sine wave values, frequency inputs, mode selector
- [ ] Graph page — uplot with amplitude_sum + individual waves over time

### Winder v2 (vendor=1, machine=0x02/2)
- [ ] Namespace (`winder2Namespace.ts`)
- [ ] Control page — spool speed, traverse, puller, tension arm state
- [ ] Settings page — traverse limits, spool diameter config
- [ ] Graph page — speed/tension over time
- [ ] Presets page
- [ ] Manual page

### Extruder v2 (vendor=1, machine=0x04/4) / Extruder v3 (machine=0x16/22)
- [ ] Namespace
- [ ] Control page — heating zones (setpoints + actual temps), inverter speed
- [ ] Settings page — PID parameters per zone
- [ ] Graph page — temperature history
- [ ] Presets page
- [ ] Manual page

### Laser v1 (vendor=1, machine=0x06/6)
- [ ] Namespace
- [ ] Control page — live diameter, target diameter, tolerance band, in-tolerance indicator
- [ ] Graph page — diameter over time with tolerance lines
- [ ] Presets page
- [ ] Manual page

### Buffer v1 (vendor=1, machine=0x08/8)
- [ ] Namespace
- [ ] Control page — motor speed setpoints, actual speeds
- [ ] Settings page

### Aquapath v1 (vendor=1, machine=0x09/9)
- [ ] Namespace
- [ ] Control page — temperature setpoints, analog outputs, encoder position
- [ ] Settings page
- [ ] Graph page

### Wago Power v1 (vendor=1, machine=0x0A/10)
- [ ] Namespace
- [ ] Control page — power channel on/off display

### Digital Input Test Machine (vendor=1, machine=0x40/64)
- [ ] Namespace + Control page — digital input state display

### Wago 8ch DIO Test (vendor=1, machine=0x41/65)
- [ ] Namespace + Control page

### Wago DO Test (vendor=1, machine=0x0E/14)
- [ ] Namespace + Control page

### Wago AI Test (vendor=1, machine=0x36/54)
- [ ] Namespace + Control page

### IP20 Test Machine (vendor=1, machine=0x34/52)
- [ ] Namespace + Control page

### Motor Test Machine (vendor=1, machine=0x11/17)
- [ ] Namespace + Control page

### Analog Input Test Machine (vendor=1, machine=0x35/53)
- [ ] Namespace + Control page

---

## Pages Outside Machine Control

- [ ] Machines overview — currently functional, but uses `unknown-*` slug for unregistered machines
- [ ] EtherCAT status page — show `ethercatDevices` and `ethercatInterface` from main namespace
- [ ] Setup page — EtherCAT interface picker (currently hardcoded in backend)
- [ ] About / version page

---

## Nice-to-Have / Polish

- [ ] Loading skeleton states (instead of plain "Waiting…" text)
- [ ] Error state display when machine.error is set
- [ ] Reconnect indicator (socket disconnect/reconnect visual feedback)
- [ ] Responsive layout for smaller screens
- [ ] Keyboard shortcuts (already exists in electron via keybindings)

---

## What to Drop from Electron

These exist in the Electron app but are **not needed** in the SolidJS version:

- Electron IPC (auto-update, NixOS config, system tray) — not applicable in browser
- `ThrottledStoreUpdater` — replaced by SolidJS `batch()`
- Zustand + Immer — replaced by `createSignal`/`createStore`
- `useStateOptimistic` — two lines inline in each component
- `createNamespaceHookImplementation` — replaced by `createNamespace()`
- `react-hook-form` — bare signals are enough for the simple forms here
- i18next — skip unless multi-language is required

---

## Testing the PoC with Mock Mode

Start the backend in mock mode:
```sh
cargo run --features mock-machine
```

Start the frontend dev server:
```sh
cd qitech-control-gui && npm run dev
```

Open http://localhost:3000 — the mock machine should appear in the sidebar.
Navigate to `/machines/mock1/{serial}/control` to see the control page.
