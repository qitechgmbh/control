import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { mock1 } from "@/machines/properties";
import { mock1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useMock1Namespace, Mode, StateEvent } from "./mock1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

function useMock(machine_identification_unique: MachineIdentificationUnique) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, sineWaveSum, sineWave1, sineWave2, sineWave3 } =
    useMock1Namespace(machine_identification_unique);

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
    serverRequest: () => Promise<any>,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest()
      .then((response) => {
        if (!response.success) stateOptimistic.resetToReal();
      })
      .catch(() => stateOptimistic.resetToReal());
  };

  // Action functions with verb-first names
  const setSchemaFrequency1 = z.object({ SetFrequency1: z.number() });
  const { request: requestSetFrequency1 } =
    useMachineMutation(setSchemaFrequency1);
  const setFrequency1 = (frequency: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.frequency1 = frequency;
      },
      () =>
        requestSetFrequency1({
          machine_identification_unique,
          data: {
            SetFrequency1: frequency,
          },
        }),
    );
  };

  const setSchemaFrequency2 = z.object({ SetFrequency2: z.number() });
  const { request: requestSetFrequency2 } =
    useMachineMutation(setSchemaFrequency2);
  const setFrequency2 = (frequency: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.frequency2 = frequency;
      },
      () =>
        requestSetFrequency2({
          machine_identification_unique,
          data: {
            SetFrequency2: frequency,
          },
        }),
    );
  };

  const setSchemaFrequency3 = z.object({ SetFrequency3: z.number() });
  const { request: requestSetFrequency3 } =
    useMachineMutation(setSchemaFrequency3);
  const setFrequency3 = (frequency: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.frequency3 = frequency;
      },
      () =>
        requestSetFrequency3({
          machine_identification_unique,
          data: {
            SetFrequency3: frequency,
          },
        }),
    );
  };

  const schemaSetMode = z.object({ SetMode: z.enum(["Standby", "Running"]) });
  const { request: requestSetMode } = useMachineMutation(schemaSetMode);
  const setMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestSetMode({
          machine_identification_unique,
          data: {
            SetMode: mode,
          },
        }),
    );
  };

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    sineWave1,
    sineWave2,
    sineWave3,
    sineWaveSum,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setFrequency1,
    setFrequency2,
    setFrequency3,
    setMode,
  };
}

export function useMock1() {
  const { serial: serialString } = mock1SerialRoute.useParams();

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
      machine_identification: mock1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const mock = useMock(machineIdentification);

  return {
    ...mock,
  };
}
