import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, extruder2 } from "@/machines/types";
import { extruder2Route } from "@/routes/routes";
import { z } from "zod";
import { Heating, Mode, useExtruder2Namespace } from "./extruder2Namespace";
import { useEffect, useMemo } from "react";
import { TimeSeries } from "@/lib/timeseries";
import { FPS_60, useThrottle } from "@/lib/useThrottle";

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
  const heatingPower = useHeatingPower(machineIdentification);
  const settings = useSettings(machineIdentification);
  return {
    ...inverter,
    ...mode,
    ...motor,
    ...heating,
    ...settings,
    ...heatingPower,
  };
}

export function useHeatingPower(
  machine_identification_unique: MachineIdentificationUnique,
): {
  nozzlePower: TimeSeries;
  frontPower: TimeSeries;
  middlePower: TimeSeries;
  backPower: TimeSeries;
} {
  const { nozzlePower, frontPower, middlePower, backPower } =
    useExtruder2Namespace(machine_identification_unique);

  return { nozzlePower, frontPower, middlePower, backPower };
}

export function useSettings(
  machine_identification_unique: MachineIdentificationUnique,
): {
  extruderSetPressureLimit: (pressure_limit: number) => void;
  extruderSetPressureLimitIsEnabled: (
    pressure_limit_is_enabled: boolean,
  ) => void;
  pressureLimitState: number | undefined;
  pressureLimitEnabledState: boolean | undefined;
} {
  const pressureLimitState = useStateOptimistic();
  const pressureLimitEnabledState = useStateOptimistic();
  // Define schemas
  const pressureLimitSchema = z.object({
    ExtruderSetPressureLimit: z.number(),
  });
  const pressureLimitEnabledSchema = z.object({
    ExtruderSetPressureLimitIsEnabled: z.boolean(),
  });

  // Create mutation hooks
  const { request: pressureLimit } = useMachineMutation(pressureLimitSchema);
  const { request: pressureLimitIsEnabled } = useMachineMutation(
    pressureLimitEnabledSchema,
  );

  const { extruderSettingsState } = useExtruder2Namespace(
    machine_identification_unique,
  );

  // Set pressure limit value
  const extruderSetPressureLimit = async (pressure: number) => {
    pressureLimitState.setOptimistic(pressure);
    pressureLimit({
      machine_identification_unique,
      data: { ExtruderSetPressureLimit: pressure },
    });
  };

  // Enable/disable pressure limit
  const extruderSetPressureLimitIsEnabled = (enabled: boolean) => {
    pressureLimitEnabledState.setOptimistic(enabled);
    pressureLimitIsEnabled({
      machine_identification_unique,
      data: { ExtruderSetPressureLimitIsEnabled: enabled },
    });
  };

  useEffect(() => {
    if (extruderSettingsState?.data) {
      pressureLimitState.setReal(extruderSettingsState.data.pressure_limit);
      pressureLimitEnabledState.setReal(
        extruderSettingsState.data.pressure_limit,
      );
    }
  }, [
    extruderSettingsState?.data.pressure_limit,
    extruderSettingsState?.data.pressure_limit_enabled,
  ]);

  return {
    extruderSetPressureLimit,
    extruderSetPressureLimitIsEnabled,
    pressureLimitState: extruderSettingsState?.data.pressure_limit,
    pressureLimitEnabledState:
      extruderSettingsState?.data.pressure_limit_enabled,
  };
}

export function useInverter(
  machine_identification_unique: MachineIdentificationUnique,
): {
  inverterSetRotation: (forward: boolean) => void;
  rotationState: boolean | undefined;
} {
  const state = useStateOptimistic();

  const schema = z.object({ InverterRotationSetDirection: z.boolean() });
  const { request: requestRotation } = useMachineMutation(schema);
  const inverterSetRotation = async (forward: boolean) => {
    state.setOptimistic(forward);
    requestRotation({
      machine_identification_unique,
      data: { InverterRotationSetDirection: forward },
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
  extruderSetMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    ExtruderSetMode: z.enum(["Heat", "Extrude", "Standby"]),
  });

  const { request } = useMachineMutation(schema);

  const extruderSetMode = async (value: Mode) => {
    state.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { ExtruderSetMode: value },
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
    extruderSetMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

export function useMotor(
  machine_identification_unique: MachineIdentificationUnique,
): {
  uses_rpm: boolean | undefined;
  rpm: TimeSeries;
  targetRpm: number | undefined;
  bar: TimeSeries;
  targetBar: number | undefined;
  screwSetTargetRpm: (rpm: number) => void;
  screwSetRegulation: (usesRpm: boolean) => void;
  screwSetTargetPressure: (bar: number) => void;
} {
  const SetTargetRpmSchema = z.object({
    InverterSetTargetRpm: z.number(),
  });

  const SetRegulationSchema = z.object({
    InverterSetRegulation: z.boolean(),
  });

  const SetTargetPressureSchema = z.object({
    InverterSetTargetPressure: z.number(),
  });

  const { motorRpmState, motorBarState, motorRegulationState, rpm, bar } =
    useExtruder2Namespace(machine_identification_unique);

  const rpmState = useStateOptimistic<number>();
  const rpmTargetState = useStateOptimistic<number>();

  const regulationState = useStateOptimistic<boolean>();
  const { request: regulationRequest } =
    useMachineMutation(SetRegulationSchema);

  const screwSetRegulation = async (value: boolean) => {
    regulationState.setOptimistic(value);
    regulationRequest({
      machine_identification_unique,
      data: { InverterSetRegulation: value },
    })
      .then((response) => {
        if (!response.success) regulationState.resetToReal();
      })
      .catch(() => regulationState.resetToReal());
  };

  const { request: reqestTargetRpm } = useMachineMutation(SetTargetRpmSchema);
  const screwSetTargetRpm = async (value: number) => {
    rpmTargetState.setOptimistic(value);
    reqestTargetRpm({
      machine_identification_unique,
      data: { InverterSetTargetRpm: value },
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

  const screwSetTargetPressure = async (value: number) => {
    targetPressureState.setOptimistic(value);
    targetPressureRequest({
      machine_identification_unique,
      data: { InverterSetTargetPressure: value },
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

  // debounce rpm and bar to 60fps
  const rpmThrottled = useThrottle(rpm, FPS_60);
  const barThrottled = useThrottle(bar, FPS_60);

  return {
    rpm: rpmThrottled,
    uses_rpm: regulationState.value,
    targetRpm: rpmTargetState.value,
    targetBar: targetPressureState.value,
    bar: barThrottled,

    screwSetTargetRpm,
    screwSetTargetPressure,
    screwSetRegulation,
  };
}

export function useHeatingTemperature(
  machine_identification_unique: MachineIdentificationUnique,
): {
  heatingSetNozzleTemp: (value: number) => void;
  heatingSetFrontTemp: (value: number) => void;
  heatingSetBackTemp: (value: number) => void;
  heatingSetMiddleTemp: (value: number) => void;

  nozzleHeatingTarget: number | undefined;
  frontHeatingTarget: number | undefined;
  backHeatingTarget: number | undefined;
  middleHeatingTarget: number | undefined;

  nozzleHeatingState: Heating | undefined;
  frontHeatingState: Heating | undefined;
  backHeatingState: Heating | undefined;
  middleHeatingState: Heating | undefined;

  nozzleTemperature: TimeSeries;
  frontTemperature: TimeSeries;
  backTemperature: TimeSeries;
  middleTemperature: TimeSeries;
} {
  const nozzleHeatingTargetState = useStateOptimistic<number>();
  const frontHeatingTargetState = useStateOptimistic<number>();
  const backHeatingTargetState = useStateOptimistic<number>();
  const middleHeatingTargetState = useStateOptimistic<number>();

  const SetNozzleHeatingSchema = z.object({
    NozzleSetHeatingTemperature: z.number(),
  });

  const SetFrontHeatingSchema = z.object({
    FrontHeatingSetTargetTemperature: z.number(),
  });

  const SetBackHeatingSchema = z.object({
    BackHeatingSetTargetTemperature: z.number(),
  });

  const SetMiddleHeatingSchema = z.object({
    MiddleSetHeatingTemperature: z.number(),
  });

  const { request: HeatingNozzleRequest } = useMachineMutation(
    SetNozzleHeatingSchema,
  );

  const heatingSetNozzleTemp = async (value: number) => {
    frontHeatingTargetState.setOptimistic(value);
    HeatingNozzleRequest({
      machine_identification_unique,
      data: { NozzleSetHeatingTemperature: value },
    })
      .then((response) => {
        if (!response.success) nozzleHeatingTargetState.resetToReal();
      })
      .catch(() => nozzleHeatingTargetState.resetToReal());
  };

  const { request: HeatiingFrontRequest } = useMachineMutation(
    SetFrontHeatingSchema,
  );

  const heatingSetFrontTemp = async (value: number) => {
    frontHeatingTargetState.setOptimistic(value);
    HeatiingFrontRequest({
      machine_identification_unique,
      data: { FrontHeatingSetTargetTemperature: value },
    })
      .then((response) => {
        if (!response.success) frontHeatingTargetState.resetToReal();
      })
      .catch(() => frontHeatingTargetState.resetToReal());
  };

  const { request: HeatingBackRequest } =
    useMachineMutation(SetBackHeatingSchema);

  const heatingSetBackTemp = async (value: number) => {
    backHeatingTargetState.setOptimistic(value);
    HeatingBackRequest({
      machine_identification_unique,
      data: { BackHeatingSetTargetTemperature: value },
    })
      .then((response) => {
        if (!response.success) backHeatingTargetState.resetToReal();
      })
      .catch(() => backHeatingTargetState.resetToReal());
  };

  const { request: HeatingMiddleRequest } = useMachineMutation(
    SetMiddleHeatingSchema,
  );

  const heatingSetMiddleTemp = async (value: number) => {
    middleHeatingTargetState.setOptimistic(value);
    HeatingMiddleRequest({
      machine_identification_unique,
      data: { MiddleSetHeatingTemperature: value },
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
    heatingNozzleState,
    frontTemperature,
    backTemperature,
    middleTemperature,
    nozzleTemperature,
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
    if (heatingNozzleState?.data) {
      nozzleHeatingTargetState.setReal(
        heatingNozzleState.data.target_temperature,
      );
    }
  }, [
    frontHeatingTargetState,
    backHeatingTargetState,
    middleHeatingTargetState,
    nozzleHeatingTargetState,
  ]);

  // debounce fast changeing values to 60fps
  const nozzleHeatingStateThrottled = useThrottle(
    heatingNozzleState?.data,
    FPS_60,
  );
  const frontHeatingStateThrottled = useThrottle(
    heatingFrontState?.data,
    FPS_60,
  );
  const backHeatingStateThrottled = useThrottle(heatingBackState?.data, FPS_60);
  const middleHeatingStateThrottled = useThrottle(
    heatingMiddleState?.data,
    FPS_60,
  );
  const nozzleTemperatureThrottled = useThrottle(nozzleTemperature, FPS_60);
  const frontTemperatureThrottled = useThrottle(frontTemperature, FPS_60);
  const backTemperatureThrottled = useThrottle(backTemperature, FPS_60);
  const middleTemperatureThrottled = useThrottle(middleTemperature, FPS_60);

  return {
    heatingSetNozzleTemp,
    heatingSetFrontTemp,
    heatingSetBackTemp,
    heatingSetMiddleTemp,
    nozzleHeatingTarget: nozzleHeatingTargetState.value,
    frontHeatingTarget: frontHeatingTargetState.value,
    backHeatingTarget: backHeatingTargetState.value,
    middleHeatingTarget: middleHeatingTargetState.value,

    nozzleHeatingState: nozzleHeatingStateThrottled,
    frontHeatingState: frontHeatingStateThrottled,
    backHeatingState: backHeatingStateThrottled,
    middleHeatingState: middleHeatingStateThrottled,

    nozzleTemperature: nozzleTemperatureThrottled,
    frontTemperature: frontTemperatureThrottled,
    backTemperature: backTemperatureThrottled,
    middleTemperature: middleTemperatureThrottled,
  };
}
