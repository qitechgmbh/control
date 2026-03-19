import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { properties } from "./properties";
import { serialRoute } from "./routes";
import { z } from "zod";
import { useNamespace, StateEvent } from "./namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

function useFF01(
  machine_identification_unique: MachineIdentificationUnique,
) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, weightPeak, weightPrev } =
    useNamespace(machine_identification_unique);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

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

  // Mutation schemas
  const { request: requestTare } = useMachineMutation(z.literal("Tare"));
  const { request: requestClearTare } = useMachineMutation(z.literal("ClearTare"));
  const { request: requestClearLights } = useMachineMutation(z.literal("ClearLights"));

  const tare = () => {
    requestTare({
      machine_identification_unique,
      data: "Tare",
    });
  };

  const clearTare = () => {
    requestClearTare({
      machine_identification_unique,
      data: "ClearTare",
    });
  };

  const clearLights = () => {
    requestClearLights({
      machine_identification_unique,
      data: "ClearLights",
    });
  };

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Live values (TimeSeries)
    weightPeak: weightPeak,
    weightPrev: weightPrev,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions
    tare,
    clearTare,
    clearLights,
  };
}

export function useFF01_v1() {
  const { serial: serialString } = serialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString); // Use 0 as fallback if NaN

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
      machine_identification: properties.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const ff01 = useFF01(machineIdentification);

  return {
    ...ff01,
  };
}
