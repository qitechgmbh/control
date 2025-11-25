import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { xtremZebra1 } from "@/machines/properties";
import { xtremZebraSerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useXtremZebraNamespace, StateEvent } from "./xtremZebraNamespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

function useXtremZebra(
  machine_identification_unique: MachineIdentificationUnique,
) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, total_weight, current_weight } =
    useXtremZebraNamespace(machine_identification_unique);

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

  // Action functions with verb-first names

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Live values (TimeSeries)
    total_weight,
    current_weight,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
  };
}

export function useXtremZebra1() {
  const { serial: serialString } = xtremZebraSerialRoute.useParams();

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
      machine_identification: xtremZebra1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const xtremZebra = useXtremZebra(machineIdentification);

  return {
    ...xtremZebra,
  };
}
