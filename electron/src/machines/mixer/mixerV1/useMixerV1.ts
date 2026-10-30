import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { mixerV1 } from "@/machines/properties";
import { mixerV1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useEffect, useMemo } from "react";
import { produce } from "immer";
import { StateEvent, useMixerV1Namespace } from "./mixerV1Namespace";

export function useMixerV1() {
  const { serial: serialString } = mixerV1SerialRoute.useParams();

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
      machine_identification: mixerV1.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state, defaultState, hopperARpm, hopperBRpm } =
    useMixerV1Namespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state]);

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

  const setMixingMotorOn = (on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.mixing_motor_state.on = on;
      },
      () =>
        requestMixingMotorOn({
          machine_identification_unique: machineIdentification,
          data: { SetMixingMotorOn: on },
        }),
    );
  };

  const setHopperAEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_a_state.enabled = enabled;
      },
      () =>
        requestHopperAEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetHopperAEnabled: enabled },
        }),
    );
  };

  const setHopperATargetRpm = (rpm: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_a_state.target_rpm = rpm;
      },
      () =>
        requestHopperATargetRpm({
          machine_identification_unique: machineIdentification,
          data: { SetHopperATargetRpm: rpm },
        }),
    );
  };

  const setHopperAForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_a_state.forward = forward;
      },
      () =>
        requestHopperAForward({
          machine_identification_unique: machineIdentification,
          data: { SetHopperAForward: forward },
        }),
    );
  };

  const setHopperADosingPercent = (percent: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_a_state.dosing_percent = percent;
      },
      () =>
        requestHopperADosingPercent({
          machine_identification_unique: machineIdentification,
          data: { SetHopperADosingPercent: percent },
        }),
    );
  };

  const setHopperACalibrationStepsPerKgh = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_a_state.calibration_steps_per_kgh = value;
      },
      () =>
        requestHopperACalibrationStepsPerKgh({
          machine_identification_unique: machineIdentification,
          data: { SetHopperACalibrationStepsPerKgh: value },
        }),
    );
  };

  const setHopperBEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_b_state.enabled = enabled;
      },
      () =>
        requestHopperBEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetHopperBEnabled: enabled },
        }),
    );
  };

  const setHopperBTargetRpm = (rpm: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_b_state.target_rpm = rpm;
      },
      () =>
        requestHopperBTargetRpm({
          machine_identification_unique: machineIdentification,
          data: { SetHopperBTargetRpm: rpm },
        }),
    );
  };

  const setHopperBForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_b_state.forward = forward;
      },
      () =>
        requestHopperBForward({
          machine_identification_unique: machineIdentification,
          data: { SetHopperBForward: forward },
        }),
    );
  };

  const setHopperBDosingPercent = (percent: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_b_state.dosing_percent = percent;
      },
      () =>
        requestHopperBDosingPercent({
          machine_identification_unique: machineIdentification,
          data: { SetHopperBDosingPercent: percent },
        }),
    );
  };

  const setHopperBCalibrationStepsPerKgh = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.hopper_b_state.calibration_steps_per_kgh = value;
      },
      () =>
        requestHopperBCalibrationStepsPerKgh({
          machine_identification_unique: machineIdentification,
          data: { SetHopperBCalibrationStepsPerKgh: value },
        }),
    );
  };

  const setExtruderKgPerRpm = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.extruder_kg_per_rpm = value;
      },
      () =>
        requestExtruderKgPerRpm({
          machine_identification_unique: machineIdentification,
          data: { SetExtruderKgPerRpm: value },
        }),
    );
  };

  const { request: requestMixingMotorOn } = useMachineMutation(
    z.object({ SetMixingMotorOn: z.boolean() }),
  );
  const { request: requestHopperAEnabled } = useMachineMutation(
    z.object({ SetHopperAEnabled: z.boolean() }),
  );
  const { request: requestHopperATargetRpm } = useMachineMutation(
    z.object({ SetHopperATargetRpm: z.number() }),
  );
  const { request: requestHopperAForward } = useMachineMutation(
    z.object({ SetHopperAForward: z.boolean() }),
  );
  const { request: requestHopperADosingPercent } = useMachineMutation(
    z.object({ SetHopperADosingPercent: z.number() }),
  );
  const { request: requestHopperACalibrationStepsPerKgh } = useMachineMutation(
    z.object({ SetHopperACalibrationStepsPerKgh: z.number() }),
  );
  const { request: requestHopperBEnabled } = useMachineMutation(
    z.object({ SetHopperBEnabled: z.boolean() }),
  );
  const { request: requestHopperBTargetRpm } = useMachineMutation(
    z.object({ SetHopperBTargetRpm: z.number() }),
  );
  const { request: requestHopperBForward } = useMachineMutation(
    z.object({ SetHopperBForward: z.boolean() }),
  );
  const { request: requestHopperBDosingPercent } = useMachineMutation(
    z.object({ SetHopperBDosingPercent: z.number() }),
  );
  const { request: requestHopperBCalibrationStepsPerKgh } = useMachineMutation(
    z.object({ SetHopperBCalibrationStepsPerKgh: z.number() }),
  );
  const { request: requestExtruderKgPerRpm } = useMachineMutation(
    z.object({ SetExtruderKgPerRpm: z.number() }),
  );

  return {
    state: stateOptimistic.value,
    defaultState,
    hopperARpm,
    hopperBRpm,

    setMixingMotorOn,
    setHopperAEnabled,
    setHopperATargetRpm,
    setHopperAForward,
    setHopperADosingPercent,
    setHopperACalibrationStepsPerKgh,
    setHopperBEnabled,
    setHopperBTargetRpm,
    setHopperBForward,
    setHopperBDosingPercent,
    setHopperBCalibrationStepsPerKgh,
    setExtruderKgPerRpm,
  };
}
