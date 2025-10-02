import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { laser1 } from "@/machines/properties";
import { laser1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useLaser1Namespace, StateEvent } from "./laser1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

function useLaser(machine_identification_unique: MachineIdentificationUnique) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, diameter, x_diameter, y_diameter, roundness } =
    useLaser1Namespace(machine_identification_unique);

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
  const schemaTargetDiameter = z.object({
    SetTargetDiameter: z.number(),
  });
  const { request: requestTargetDiameter } =
    useMachineMutation(schemaTargetDiameter);

  const schemaLowerTolerance = z.object({
    SetLowerTolerance: z.number(),
  });
  const { request: requestLowerTolerance } =
    useMachineMutation(schemaLowerTolerance);

  const schemaHigherTolerance = z.object({
    SetHigherTolerance: z.number(),
  });
  const { request: requestHigherTolerance } = useMachineMutation(
    schemaHigherTolerance,
  );

  // Action functions with verb-first names
  const setTargetDiameter = (target_diameter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.target_diameter = target_diameter;
      },
      () =>
        requestTargetDiameter({
          machine_identification_unique,
          data: {
            SetTargetDiameter: target_diameter,
          },
        }),
    );
  };

  const setLowerTolerance = (lower_tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.lower_tolerance = lower_tolerance;
      },
      () =>
        requestLowerTolerance({
          machine_identification_unique,
          data: {
            SetLowerTolerance: lower_tolerance,
          },
        }),
    );
  };

  const setHigherTolerance = (higher_tolerance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.laser_state.higher_tolerance = higher_tolerance;
      },
      () =>
        requestHigherTolerance({
          machine_identification_unique,
          data: {
            SetHigherTolerance: higher_tolerance,
          },
        }),
    );
  };

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Live values (TimeSeries)
    diameter,
    x_diameter,
    y_diameter,
    roundness,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
  };
}

export function useLaser1() {
  const { serial: serialString } = laser1SerialRoute.useParams();

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
      machine_identification: laser1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const laser = useLaser(machineIdentification);

  return {
    ...laser,
  };
}
