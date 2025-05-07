import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, extruder2 } from "@/machines/types";
import { extruder2Route } from "@/routes/routes";
import { z } from "zod";
import {
  Heating,
  Mode,
  MotorPressure,
  heatingStateDataSchema,
  useExtruder2Namespace,
} from "./extruder2Namespace";
import { useEffect, useMemo, useState } from "react";
import { TimeSeries } from "@/lib/timeseries";

export function useExtruder2() {
  const { serial: serialString } = extruder2Route.useParams();

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
      machine_identification: extruder2.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const inverter = useInverter(machineIdentification);
  const mode = useMode(machineIdentification);
  const motor = useMotor(machineIdentification);
  const heating = useHeatingTemperature(machineIdentification);

  return {
    ...inverter,
    ...mode,
    ...motor,
    ...heating,
  };
}

export function useInverter(
  machine_identification_unique: MachineIdentificationUnique,
): {
  inverterSetRotation: (forward: boolean) => void;
  rotationState: boolean | undefined;
} {
  const state = useStateOptimistic();

  const schema = z.object({ SetRotation: z.boolean() });
  const { request: requestRotation } = useMachineMutation(schema);
  const inverterSetRotation = async (forward: boolean) => {
    state.setOptimistic(forward);
    requestRotation({
      machine_identification_unique,
      data: { SetRotation: forward },
    });
  };

  const { rotationState } = useExtruder2Namespace(
    machine_identification_unique,
  );

  useEffect(() => {
    if (rotationState?.data) {
      state.setReal(rotationState.data.forward);
    }
  }, [rotationState?.data.forward]);

  return { inverterSetRotation, rotationState: rotationState?.data.forward };
}

export function useMode(
  machine_identification_unique: MachineIdentificationUnique,
): {
  mode: Mode | undefined;
  SetMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    SetMode: z.enum(["Heat", "Extrude", "Standby"]),
  });

  const { request } = useMachineMutation(schema);

  const SetMode = async (value: Mode) => {
    state.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { SetMode: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  // Read path
  const { modeState } = useExtruder2Namespace(machine_identification_unique);
  useEffect(() => {
    if (modeState?.data) {
      state.setReal(modeState.data.mode);
    }
  }, [modeState]);

  return {
    mode: state.value,
    SetMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

export function useMotor(
  machine_identification_unique: MachineIdentificationUnique,
): {
  rpm: number | undefined;
  bar: number | undefined;
  targetBar: number | undefined;
  targetRpm: number | undefined;
  uses_rpm: boolean | undefined;
  rpmTs: TimeSeries;
  barTs: TimeSeries;
  SetTargetRpm: (rpm: number) => void;
  SetRegulation: (usesRpm: boolean) => void;
  SetTargetPressure: (bar: number) => void;
} {
  const SetTargetRpmSchema = z.object({
    SetTargetRpm: z.number(),
  });

  const SetRegulationSchema = z.object({
    SetRegulation: z.boolean(),
  });

  const SetTargetPressureSchema = z.object({
    SetTargetPressure: z.number(),
  });

  const { motorRpmState, motorBarState, motorRegulationState, rpm, bar } =
    useExtruder2Namespace(machine_identification_unique);

  const rpmState = useStateOptimistic<number>();
  const rpmTargetState = useStateOptimistic<number>();

  const regulationState = useStateOptimistic<boolean>();
  const { request: regulationRequest } =
    useMachineMutation(SetRegulationSchema);

  const SetRegulation = async (value: boolean) => {
    regulationState.setOptimistic(value);
    regulationRequest({
      machine_identification_unique,
      data: { SetRegulation: value },
    })
      .then((response) => {
        if (!response.success) regulationState.resetToReal();
      })
      .catch(() => regulationState.resetToReal());
  };

  const { request: reqestTargetRpm } = useMachineMutation(SetTargetRpmSchema);
  const SetTargetRpm = async (value: number) => {
    rpmTargetState.setOptimistic(value);
    reqestTargetRpm({
      machine_identification_unique,
      data: { SetTargetRpm: value },
    })
      .then((response) => {
        if (!response.success) rpmTargetState.resetToReal();
      })
      .catch(() => rpmTargetState.resetToReal());
  };

  const pressureState = useStateOptimistic<number>();
  const targetPressureState = useStateOptimistic<number>();

  const { request: targetPressureRequest } = useMachineMutation(
    SetTargetPressureSchema,
  );

  const SetTargetPressure = async (value: number) => {
    targetPressureState.setOptimistic(value);
    targetPressureRequest({
      machine_identification_unique,
      data: { SetTargetPressure: value },
    })
      .then((response) => {
        if (!response.success) targetPressureState.resetToReal();
      })
      .catch(() => targetPressureState.resetToReal());
  };

  useEffect(() => {
    if (motorRpmState?.data) {
      rpmState.setReal(motorRpmState.data.rpm);
      rpmTargetState.setReal(motorRpmState.data.target_rpm);
    }

    if (motorBarState?.data) {
      pressureState.setReal(motorBarState.data.bar);
      targetPressureState.setReal(motorBarState.data.target_bar);
    }

    if (motorRegulationState?.data) {
      regulationState.setReal(motorRegulationState.data.uses_rpm);
    }
  }, [motorRpmState, motorBarState, motorRegulationState]);

  return {
    uses_rpm: regulationState.value,
    rpm: rpmState.value,
    bar: pressureState.value,
    targetBar: targetPressureState.value,
    targetRpm: rpmTargetState.value,
    rpmTs: rpm,
    barTs: bar,
    SetTargetRpm,
    SetRegulation,
    SetTargetPressure,
  };
}

export function useHeatingTemperature(
  machine_identification_unique: MachineIdentificationUnique,
): {
  SetHeatingFrontTemp: (value: number) => void;
  SetHeatingBackTemp: (value: number) => void;
  SetHeatingMiddleTemp: (value: number) => void;

  frontHeatingTarget: number | undefined;
  backHeatingTarget: number | undefined;
  middleHeatingTarget: number | undefined;

  frontHeatingState: Heating | undefined;
  backHeatingState: Heating | undefined;
  middleHeatingState: Heating | undefined;

  frontTemperature: TimeSeries;
  backTemperature: TimeSeries;
  middleTemperature: TimeSeries;
} {
  const frontHeatingTargetState = useStateOptimistic<number>();
  const backHeatingTargetState = useStateOptimistic<number>();
  const middleHeatingTargetState = useStateOptimistic<number>();

  const SetFrontHeatingSchema = z.object({
    SetFrontHeatingTemperature: z.number(),
  });

  const SetBackHeatingSchema = z.object({
    SetBackHeatingTemperature: z.number(),
  });

  const SetMiddleHeatingSchema = z.object({
    SetMiddleHeatingTemperature: z.number(),
  });

  const { request: HeatiingFrontRequest } = useMachineMutation(
    SetFrontHeatingSchema,
  );
  const SetHeatingFrontTemp = async (value: number) => {
    frontHeatingTargetState.setOptimistic(value);
    HeatiingFrontRequest({
      machine_identification_unique,
      data: { SetFrontHeatingTemperature: value },
    })
      .then((response) => {
        if (!response.success) frontHeatingTargetState.resetToReal();
      })
      .catch(() => frontHeatingTargetState.resetToReal());
  };

  const { request: HeatingBackRequest } =
    useMachineMutation(SetBackHeatingSchema);

  const SetHeatingBackTemp = async (value: number) => {
    backHeatingTargetState.setOptimistic(value);
    HeatingBackRequest({
      machine_identification_unique,
      data: { SetBackHeatingTemperature: value },
    })
      .then((response) => {
        if (!response.success) backHeatingTargetState.resetToReal();
      })
      .catch(() => backHeatingTargetState.resetToReal());
  };

  const { request: HeatingMiddleRequest } = useMachineMutation(
    SetMiddleHeatingSchema,
  );

  const SetHeatingMiddleTemp = async (value: number) => {
    middleHeatingTargetState.setOptimistic(value);
    HeatingMiddleRequest({
      machine_identification_unique,
      data: { SetMiddleHeatingTemperature: value },
    })
      .then((response) => {
        if (!response.success) middleHeatingTargetState.resetToReal();
      })
      .catch(() => middleHeatingTargetState.resetToReal());
  };

  // Read path
  const {
    heatingFrontState,
    heatingBackState,
    heatingMiddleState,
    frontTemperature,
    backTemperature,
    middleTemperature,
  } = useExtruder2Namespace(machine_identification_unique);

  useEffect(() => {
    if (heatingFrontState?.data) {
      frontHeatingTargetState.setReal(
        heatingFrontState.data.target_temperature,
      );
    }
    if (heatingBackState?.data) {
      backHeatingTargetState.setReal(heatingBackState.data.target_temperature);
    }
    if (heatingMiddleState?.data) {
      middleHeatingTargetState.setReal(
        heatingMiddleState.data.target_temperature,
      );
    }
  }, [
    frontHeatingTargetState,
    backHeatingTargetState,
    middleHeatingTargetState,
  ]);

  return {
    SetHeatingFrontTemp,
    SetHeatingBackTemp,
    SetHeatingMiddleTemp,

    frontHeatingTarget: frontHeatingTargetState.value,
    backHeatingTarget: backHeatingTargetState.value,
    middleHeatingTarget: middleHeatingTargetState.value,

    frontHeatingState: heatingFrontState?.data,
    backHeatingState: heatingBackState?.data,
    middleHeatingState: heatingMiddleState?.data,

    frontTemperature,
    backTemperature,
    middleTemperature,
  };
}
