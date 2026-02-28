// ============================================================================
// useMyMachine.ts — The primary React hook for this machine
// ============================================================================
// This hook is the single point of contact for components.  It:
//   1. Reads the serial number from the route params
//   2. Builds the MachineIdentificationUnique
//   3. Subscribes to backend state via the namespace hook
//   4. Applies optimistic updates so the UI feels instant
//   5. Sends mutations to the backend
//
// Components should only import from this file, not from MyMachineNamespace.ts
//
// FIND & REPLACE to adapt this template:
//   MyMachine             → YourMachineName
//   myMachine             → yourMachineName
//   myMachineSerialRoute  → the route export from routes.tsx  (see step 8 in README)
//   myMachine (properties)→ the export from properties.ts     (see step 7 in README)
// ============================================================================

import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
// TODO: Import the actual serial route.
//import { myMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMyMachineNamespace, StateEvent } from "./MyMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
// TODO: Import the actual properties.
//import { myMachine } from "@/machines/properties";
import { z } from "zod";

export function useMyMachine() {
  // --------------------------------------------------------------------------
  // 1. Route params — serial comes from the URL (e.g. /machines/mymachine/42)
  // --------------------------------------------------------------------------
  // TODO: use myMachineSerialRoute.useParams();
  const { serial: serialString } = { serial: "REPLACEME" };

  // --------------------------------------------------------------------------
  // 2. Build MachineIdentificationUnique
  //    vendor + machine ID uniquely identify the machine type;
  //    serial distinguishes multiple instances of the same type.
  // --------------------------------------------------------------------------
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );
      return {
        machine_identification: { vendor: 0, machine: 0 },
        serial: 0,
      };
    }

    return {
      //TODO: use the actual myMachine.machine_identification,
      machine_identification: {
        vendor: Number.NaN,
        machine: Number.NaN,
      },
      serial,
    };
  }, [serialString]);

  // --------------------------------------------------------------------------
  // 3. Subscribe to backend state via WebSocket namespace
  // --------------------------------------------------------------------------
  const { state } = useMyMachineNamespace(machineIdentification);

  // --------------------------------------------------------------------------
  // 4. Optimistic state — mirrors real state but allows instant local updates.
  //    The UI reads `stateOptimistic.value` instead of raw `state`.
  // --------------------------------------------------------------------------
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  // --------------------------------------------------------------------------
  // 5. Mutation sender — sends JSON to the backend api_mutate handler.
  //    The schema here validates what we send (use z.any() for flexibility).
  // --------------------------------------------------------------------------
  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any(),
    }),
  );

  // --------------------------------------------------------------------------
  // Helper: apply an optimistic update locally and optionally fire a request.
  // --------------------------------------------------------------------------
  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  // --------------------------------------------------------------------------
  // 6. Mutation actions — one function per Rust `Mutation` variant.
  //
  // The `action` string and `value` shape must exactly match the
  // `#[serde(tag = "action", content = "value")]` enum in api.rs.
  //
  // TODO: replace/extend these examples with your actual mutations.
  //       For read-only machines, delete all mutation functions and the
  //       sendMutation declaration above.
  // --------------------------------------------------------------------------

  // Example: toggle a single output
  // const setOutput = (index: number, on: boolean) => {
  //   updateStateOptimistically(
  //     (current) => { current.outputs[index] = on; },
  //     () => sendMutation({
  //       machine_identification_unique: machineIdentification,
  //       data: { action: "SetOutput", value: { index, on } },
  //     }),
  //   );
  // };

  // Example: set all outputs at once
  // const setAllOutputs = (on: boolean) => {
  //   updateStateOptimistically(
  //     (current) => { current.outputs = current.outputs.map(() => on); },
  //     () => sendMutation({
  //       machine_identification_unique: machineIdentification,
  //       data: { action: "SetAllOutputs", value: { on } },
  //     }),
  //   );
  // };

  // --------------------------------------------------------------------------
  // Return everything the components need.
  // --------------------------------------------------------------------------
  return {
    state: stateOptimistic.value,
    // TODO: expose your mutation functions here, e.g.:
    //   setOutput,
    //   setAllOutputs,
  };
}
