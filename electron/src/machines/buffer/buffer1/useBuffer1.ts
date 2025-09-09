import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { buffer1, VENDOR_QITECH } from "@/machines/properties";
import {
  machineIdentificationUnique,
  MachineIdentificationUnique,
} from "@/machines/types";
import { buffer1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useEffect, useMemo } from "react";
import { StateEvent, Mode, useBuffer1Namespace } from "./buffer1Namespace";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";

export function useBuffer1() {
  const { serial: serialString } = buffer1SerialRoute.useParams();

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
      machine_identification: buffer1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  // Get machine identification unique
  const machine_identification_unique = machineIdentification;

  // Get consolidated state and live values from namespace
  const { state } = useBuffer1Namespace(machineIdentification);

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

  // Action functions with verb-first names
  const setBufferMode = async (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestBufferMode({
          machine_identification_unique: machineIdentification,
          data: { SetBufferMode: mode },
        }),
    );
  };

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

  // Mutation hooks
  const { request: requestBufferMode } = useMachineMutation(
    z.object({
      SetBufferMode: z.enum(["Standby", "FillingBuffer", "EmptyingBuffer"]),
    }),
  );

  const { request: requestConnectedMachine } = useMachineMutation(
    z.object({
      SetConnectedMachine: machineIdentificationUnique,
    }),
  );

  const { request: requestDisconnectedMachine } = useMachineMutation(
    z.object({
      DisconnectMachine: machineIdentificationUnique,
    }),
  );

  // General Helper functions
  const machines = useMachines();
  // Filter machines for the correct type
  const filteredMachines = useMemo(
    () =>
      machines.filter(
        (m) =>
          m.machine_identification_unique.machine_identification.vendor ===
            VENDOR_QITECH &&
          m.machine_identification_unique.machine_identification.machine ===
            0x0002,
      ),
    [machines, machineIdentification],
  );

  // Get selected machine by serial
  const selectedMachine = useMemo(() => {
    const serial =
      state?.data.connected_machine_state?.machine_identification_unique
        ?.serial;

    return (
      filteredMachines.find(
        (m) => m.machine_identification_unique.serial === serial,
      ) ?? null
    );
  }, [filteredMachines, state]);

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    filteredMachines,
    selectedMachine,

    // Individual live values (TimeSeries)

    // Action functions (verb-first)
    setBufferMode,
    setConnectedMachine,
    disconnectMachine,
  };
}
