import { toastError } from "@/components/Toast";
import { MachineIdentificationUnique } from "@/machines/types";
import { useEffect, useMemo } from "react";
import {
  ChannelConfig,
  StateEvent,
  WaveformType,
  useAnalogOutOversamplingNamespace,
} from "./analogOutOversamplingNamespace";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { analogOutOversamplingSerialRoute } from "@/routes/routes";
import { z } from "zod";
import { produce } from "immer";
import { useMachineMutate } from "@/client/useClient";
import { analogOutOversamplingMachine } from "@/machines/properties";

export function useAnalogOutOversampling() {
  const { serial: serialString } =
    analogOutOversamplingSerialRoute.useParams();

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
      machine_identification:
        analogOutOversamplingMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state, liveValues } = useAnalogOutOversamplingNamespace(
    machineIdentification,
  );

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any().optional(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  const setChannelConfig = (channel: number, config: ChannelConfig) => {
    updateStateOptimistically(
      (current) => {
        current.channels[channel] = config;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetChannelConfig", value: { channel, config } },
        }),
    );
  };

  const setWaveform = (channel: number, waveform: WaveformType) => {
    updateStateOptimistically(
      (current) => {
        current.channels[channel].waveform = waveform;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetWaveform", value: { channel, waveform } },
        }),
    );
  };

  const setFrequency = (channel: number, frequency_hz: number) => {
    updateStateOptimistically(
      (current) => {
        current.channels[channel].frequency_hz = frequency_hz;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetFrequency", value: { channel, frequency_hz } },
        }),
    );
  };

  const setAmplitude = (channel: number, amplitude: number) => {
    updateStateOptimistically(
      (current) => {
        current.channels[channel].amplitude = amplitude;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAmplitude", value: { channel, amplitude } },
        }),
    );
  };

  const setOffset = (channel: number, offset: number) => {
    updateStateOptimistically(
      (current) => {
        current.channels[channel].offset = offset;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetOffset", value: { channel, offset } },
        }),
    );
  };

  const isDisabled = !stateOptimistic.isInitialized;
  const isLoading = stateOptimistic.isOptimistic;

  return {
    state: stateOptimistic.value,
    liveValues,
    setChannelConfig,
    setWaveform,
    setFrequency,
    setAmplitude,
    setOffset,
    isDisabled,
    isLoading,
  };
}
