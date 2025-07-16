import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import {
  machineIdentification,
  machineIdentificationUnique,
  MachineIdentificationUnique,
} from "@/machines/types";
import { mock1, VENDOR_QITECH } from "@/machines/properties";
import { mock1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useMock1Namespace, Mode, StateEvent } from "./mock1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";

function useMock(machine_identification_unique: MachineIdentificationUnique) {
  // Get consolidated state and live values from namespace
  const { state, defaultState, sineWave } = useMock1Namespace(
    machine_identification_unique,
  );

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
    serverRequest: () => Promise<any>,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest()
      .then((response) => {
        if (!response.success) stateOptimistic.resetToReal();
      })
      .catch(() => stateOptimistic.resetToReal());
  };

  // Action functions with verb-first names
  const schemaSetFrequency = z.object({ SetFrequency: z.number() });
  const { request: requestSetFrequency } =
    useMachineMutation(schemaSetFrequency);
  const setFrequency = (frequency: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.sine_wave_state.frequency = frequency;
      },
      () =>
        requestSetFrequency({
          machine_identification_unique,
          data: {
            SetFrequency: frequency,
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

  const { request: requestConnectedMachine } = useMachineMutation(
    z.object({
      SetConnectedMachine: machineIdentificationUnique,
    }),
  );
  const setConnectedMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          machineIdentificationUnique;
      },
      () =>
        requestConnectedMachine({
          machine_identification_unique,
          data: { SetConnectedMachine: machineIdentificationUnique },
        }),
    );
  };

  const { request: requestDisconnectedMachine } = useMachineMutation(
    z.object({
      DisconnectMachine: machineIdentificationUnique,
    }),
  );
  const disconnectMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          null;
      },
      () =>
        requestDisconnectedMachine({
          machine_identification_unique,
          data: { DisconnectMachine: machineIdentificationUnique },
        }),
    );
  };

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    sineWave,

    // Loading states
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,

    // Action functions (verb-first)
    setFrequency,
    setMode,
    setConnectedMachine,
    disconnectMachine,
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

  const machines = useMachines();
  // Filter machines for the correct type
  const filteredMachines = useMemo(
    () =>
      machines.filter(
        (m) =>
          m.machine_identification_unique.machine_identification.vendor ===
            VENDOR_QITECH &&
          m.machine_identification_unique.machine_identification.machine ===
            0x0009,
      ),
    [machines, machineIdentification],
  );

  // Get selected machine by serial
  const selectedMachine = useMemo(() => {
    const serial =
      mock.state?.connected_machine_state?.machine_identification_unique
        ?.serial;

    return (
      filteredMachines.find(
        (m) => m.machine_identification_unique.serial === serial,
      ) ?? null
    );
  }, [filteredMachines, mock.state]);

  return {
    ...mock,
    filteredMachines,
    selectedMachine,
  };
}
