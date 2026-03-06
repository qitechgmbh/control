import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { wago750_553MachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useWago750_553MachineNamespace,
  StateEvent,
} from "./Wago750_553MachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo, useRef } from "react";
import { wago750_553Machine } from "@/machines/properties";
import { z } from "zod";

export function useWago750_553Machine() {
  const { serial: serialString } = wago750_553MachineSerialRoute.useParams();

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
      machine_identification: wago750_553Machine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago750_553MachineNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any(),
    }),
  );

  const debounceTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const debounced = (key: string, fn: () => void, delay = 150) => {
    const existing = debounceTimers.current.get(key);
    if (existing) clearTimeout(existing);
    debounceTimers.current.set(
      key,
      setTimeout(() => {
        fn();
        debounceTimers.current.delete(key);
      }, delay),
    );
  };

  const setOutput = (index: number, value: number) => {
    debounced(`output-${index}`, () =>
      sendMutation({
        machine_identification_unique: machineIdentification,
        data: { action: "SetOutput", value: { index, value } },
      }),
    );
  };

  const setAllOutputs = (value: number) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, (draft) => {
        draft.outputs = Array(4).fill(value);
        draft.outputs_ma = Array(4).fill(value * 20.0);
      }));
    debounced("all", () =>
      sendMutation({
        machine_identification_unique: machineIdentification,
        data: { action: "SetAllOutputs", value: { value } },
      }),
    );
  };

  return {
    state: stateOptimistic.value,
    setOutput,
    setAllOutputs,
  };
}
