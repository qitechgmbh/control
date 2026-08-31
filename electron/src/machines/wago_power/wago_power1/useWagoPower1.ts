import { toastError } from "@/components/Toast";
import { wagoPower1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { wagoPower1SerialRoute } from "@/routes/routes";
import { useEffect, useMemo } from "react";
import {
  Mode,
  modeSchema,
  StateEvent,
  useWagoPower1Namespace,
} from "./wagoPower1Namespace";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import z from "zod";
import { useMachineMutate } from "@/client/useClient";

export function useWagoPower1() {
  const { serial: serialString } = wagoPower1SerialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: {
          vendor: 0,
          machine: 0,
        },
        serial: 0,
      };
    }

    return {
      machine_identification: wagoPower1.machine_identification,
      serial,
    };
  }, [serialString]);

  const { request: requestModeSet } = useMachineMutate(
    z.object({ SetMode: modeSchema }),
  );

  const { state, defaultState, voltage, current } = useWagoPower1Namespace(
    machineIdentification,
  );
  //
  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // Helper function for optimistic updates using produce
  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  const setMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode = mode;
      },
      () =>
        requestModeSet({
          machine_identification_unique: machineIdentification,
          data: { SetMode: mode },
        }),
    );
  };

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  return {
    state: stateOptimistic.value?.data,
    defaultState: defaultState?.data,

    current,
    voltage,

    setMode,

    isLoading,
    isDisabled,
  };
}
