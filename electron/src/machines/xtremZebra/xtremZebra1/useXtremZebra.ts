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
  const {
    state,
    defaultState,
    total_weight,
    current_weight,
    plate1_counter,
    plate2_counter,
    plate3_counter,
  } = useXtremZebraNamespace(machine_identification_unique);

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
  const schemaTolerance = z.object({
    SetTolerance: z.number(),
  });
  const { request: requestTolerance } = useMachineMutation(schemaTolerance);
  const schemaPlate1Target = z.object({
    SetPlate1Target: z.number(),
  });
  const { request: requestPlate1Target } =
    useMachineMutation(schemaPlate1Target);
  const schemaPlate2Target = z.object({
    SetPlate2Target: z.number(),
  });
  const { request: requestPlate2Target } =
    useMachineMutation(schemaPlate2Target);
  const schemaPlate3Target = z.object({
    SetPlate3Target: z.number(),
  });
  const { request: requestPlate3Target } =
    useMachineMutation(schemaPlate3Target);

  const { request: requestSetTare } = useMachineMutation(z.literal("SetTare"));
  const { request: requestZeroCounters } = useMachineMutation(
    z.literal("ZeroCounters"),
  );

  const setTolerance = (tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.xtrem_zebra_state.tolerance = tolerance;
      },
      () =>
        requestTolerance({
          machine_identification_unique,
          data: {
            SetTolerance: tolerance,
          },
        }),
    );
  };

  const setPlate1Target = (plate1_target: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.xtrem_zebra_state.plate1_target = plate1_target;
      },
      () =>
        requestPlate1Target({
          machine_identification_unique,
          data: {
            SetPlate1Target: plate1_target,
          },
        }),
    );
  };

  const setPlate2Target = (plate2_target: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.xtrem_zebra_state.plate2_target = plate2_target;
      },
      () =>
        requestPlate2Target({
          machine_identification_unique,
          data: {
            SetPlate2Target: plate2_target,
          },
        }),
    );
  };

  const setPlate3Target = (plate3_target: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.xtrem_zebra_state.plate3_target = plate3_target;
      },
      () =>
        requestPlate3Target({
          machine_identification_unique,
          data: {
            SetPlate3Target: plate3_target,
          },
        }),
    );
  };

  const setTare = () => {
    requestSetTare({
      machine_identification_unique,
      data: "SetTare",
    });
  };

  const zeroCounters = () => {
    requestZeroCounters({
      machine_identification_unique,
      data: "ZeroCounters",
    });
  };

  // Action functions with verb-first names

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Live values (TimeSeries)
    total_weight,
    current_weight,
    plate1_counter,
    plate2_counter,
    plate3_counter,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setTolerance,
    setPlate1Target,
    setPlate2Target,
    setPlate3Target,
    setTare,
    zeroCounters,
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
