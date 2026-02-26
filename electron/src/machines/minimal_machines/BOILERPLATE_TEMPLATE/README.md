# Minimal Machine Frontend Boilerplate

This directory is a copy-paste template for creating the React/TypeScript frontend
for a new minimal machine.  Follow the steps below in order.  Each file has inline
`// TODO:` comments that mark every place you need to edit.

Pair this with the backend `BOILERPLATE_TEMPLATE` in
`machines/src/minimal_machines/BOILERPLATE_TEMPLATE/`.

---

## Quick-start checklist

### 1 — Copy this directory

```
cp -r BOILERPLATE_TEMPLATE MyNewMachine
```

### 2 — Rename throughout the template

Do a project-wide find & replace inside your new directory:

| Find                      | Replace with                              |
|---------------------------|-------------------------------------------|
| `MyMachine`               | `YourMachineName` (PascalCase)            |
| `myMachine`               | `yourMachineName` (camelCase)             |
| `myMachineSerialRoute`    | the route export you add in step 8        |
| `mymachine`               | the URL segment you choose in step 8      |

### 3 — Fill in `MyMachineNamespace.ts` — event schema

Open `MyMachineNamespace.ts` and add fields to `stateEventDataSchema` that
mirror your Rust `StateEvent` struct in `api.rs`:

```typescript
export const stateEventDataSchema = z.object({
  outputs: z.array(z.boolean()).length(4),
  value:   z.number(),
});
```

### 4 — Fill in `useMyMachine.ts` — mutations

Open `useMyMachine.ts` and uncomment / add mutation functions for every
`Mutation` variant in your Rust `api.rs`.  Each mutation should:

1. Call `updateStateOptimistically` with an immer producer to update local state instantly
2. Call `sendMutation` with `{ action: "VariantName", value: { ... } }` to inform the backend

```typescript
const setOutput = (index: number, on: boolean) => {
  updateStateOptimistically(
    (current) => { current.outputs[index] = on; },
    () => sendMutation({
      machine_identification_unique: machineIdentification,
      data: { action: "SetOutput", value: { index, on } },
    }),
  );
};
```

Then expose the function in the return object at the bottom of the hook.

### 5 — Fill in `MyMachineControlPage.tsx` — UI

Open `MyMachineControlPage.tsx` and:

- Update `safeState` default to match your `StateEvent` shape
- Replace the placeholder `ControlCard` with real controls
- Destructure your mutation functions from `useMyMachine()`

Common control components:

| Component        | Use for                              |
|------------------|--------------------------------------|
| `SelectionGroup` | Radio-style on/off or multi-option   |
| `Label`          | Adds a text label to any control     |
| `ControlCard`    | Groups related controls in a card    |
| `ControlGrid`    | Arranges cards in a responsive grid  |
| `Slider`         | Numeric range input                  |
| `Toggle`         | Simple boolean toggle switch         |
| `NumericInput`   | Free-form number input               |

### 6 — Fill in `MyMachinePage.tsx` — tabs

Update the `pathname` and `items` array if you add more tabs beyond "Control".
Usually this file needs no changes for simple machines.

### 7 — Add to `properties.ts`

Open `electron/src/machines/properties.ts` and add an entry for your machine:

```typescript
export const myMachine: MachineProperties = {
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: MY_MACHINE_ID,   // must match the Rust constant
  },
  name: "My Machine",
  slug: "mymachine",
  device_roles: [
    { role: 1, description: "EL2004 — 4× digital output" },
  ],
};
```

### 8 — Add to `routes.tsx`

Open `electron/src/routes/routes.tsx`.

**a) Import your page components** (at the top of the file):

```typescript
import { MyMachinePage } from "@/machines/minimal_machines/MyNewMachine/MyMachinePage";
import { MyMachineControlPage } from "@/machines/minimal_machines/MyNewMachine/MyMachineControlPage";
```

**b) Create a serial route** (follow the existing pattern):

```typescript
export const myMachineSerialRoute = createRoute({
  getParentRoute: () => machinesRoute,
  path: "mymachine/$serial",
  component: MyMachinePage,
});
```

**c) Create a control sub-route**:

```typescript
const myMachineControlRoute = createRoute({
  getParentRoute: () => myMachineSerialRoute,
  path: "control",
  component: MyMachineControlPage,
});
```

**d) Wire up children**:

```typescript
const myMachineRoutes = myMachineSerialRoute.addChildren([myMachineControlRoute]);
```

**e) Add to `routeTree`** — include `myMachineRoutes` in the top-level `addChildren` call.

### 9 — Verify the build

```
npm run typecheck   # or: tsc --noEmit
```

---

## File overview

| File                       | Responsibility                                           |
|----------------------------|----------------------------------------------------------|
| `MyMachineNamespace.ts`    | Zod schema, Zustand store, socket.io event handler, hook |
| `useMyMachine.ts`          | Route params, machine ID, state subscription, mutations  |
| `MyMachinePage.tsx`        | Topbar navigation tabs                                   |
| `MyMachineControlPage.tsx` | Control UI — cards, buttons, sliders                     |

## Data flow

```
Backend (Rust)
      │  socket.io "StateEvent"
      ▼
MyMachineNamespace.ts
  myMachineMessageHandler()  ──► Zustand store
      │
      ▼
useMyMachine.ts
  useMyMachineNamespace()     ──► real state
  useStateOptimistic()        ──► optimistic state (instant UI)
  useMachineMutate()          ──► sends JSON to backend api_mutate
      │
      ▼
MyMachineControlPage.tsx
  reads: state (optimistic)
  writes: mutation functions
```

## Naming conventions

| Concept             | Convention    | Example                         |
|---------------------|---------------|---------------------------------|
| Namespace file      | `PascalCase`  | `MyMachineNamespace.ts`         |
| Hook file           | `camelCase`   | `useMyMachine.ts`               |
| Page component      | `PascalCase`  | `MyMachinePage.tsx`             |
| Control page        | `PascalCase`  | `MyMachineControlPage.tsx`      |
| Directory           | `lowercase`   | `mymachine/`                    |
| Route serial export | `camelCase`   | `myMachineSerialRoute`          |
| Properties export   | `camelCase`   | `myMachine`                     |
