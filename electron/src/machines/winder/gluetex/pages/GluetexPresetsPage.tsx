import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import { gluetex } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import {
  pullerStateSchema,
  spoolSpeedControllerStateSchema,
  heatingStatesSchema,
  addonMotorStateSchema,
  addonMotor5StateSchema,
  slavePullerStateSchema,
  spoolAutomaticActionStateSchema,
} from "../state/gluetexNamespace";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";

const gluetexPresetDataSchema = z
  .object({
    traverse_state: z
      .object({
        limit_inner: z.number(),
        limit_outer: z.number(),
        step_size: z.number(),
        padding: z.number(),
        laserpointer: z.boolean(),
      })
      .partial(),
    puller_state: pullerStateSchema.partial(),
    spool_speed_controller_state: spoolSpeedControllerStateSchema.partial(),
    spool_automatic_action_state: spoolAutomaticActionStateSchema.partial(),
    heating_states: z
      .object({
        enabled: z.boolean(),
        zone_1_target: z.number(),
        zone_2_target: z.number(),
        zone_3_target: z.number(),
        zone_4_target: z.number(),
        zone_5_target: z.number(),
        zone_6_target: z.number(),
      })
      .partial(),
    addon_motor_3_state: addonMotor5StateSchema.partial(),
    addon_motor_4_state: addonMotorStateSchema.partial(),
    addon_motor_5_state: addonMotorStateSchema.partial(),
    slave_puller_state: slavePullerStateSchema
      .omit({ tension_arm: true })
      .partial(),
  })
  .partial();

type GluetexPresetData = z.infer<typeof gluetexPresetDataSchema>;

const schemas = new Map([[1, gluetexPresetDataSchema]]);

const previewEntries: PresetPreviewEntries<GluetexPresetData> = [
  // ── Traverse ──
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (data) =>
      data.traverse_state?.limit_inner?.toFixed(1) ?? "N/A",
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (data) =>
      data.traverse_state?.limit_outer?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Step Size",
    unit: "mm",
    renderValue: (data) => data.traverse_state?.step_size?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (data) => data.traverse_state?.padding?.toFixed(1),
  },
  previewSeparator,

  // ── Puller ──
  {
    name: "Puller Regulation",
    renderValue: (data) => data.puller_state?.regulation,
  },
  {
    name: "Puller Direction",
    renderValue: (data) =>
      data.puller_state?.forward !== undefined
        ? data.puller_state.forward
          ? "Forward"
          : "Backward"
        : "N/A",
  },
  {
    name: "Puller Gear Ratio",
    renderValue: (data) => {
      const ratio = data.puller_state?.gear_ratio;
      if (ratio === "OneToOne") return "1:1";
      if (ratio === "OneToFive") return "1:5";
      if (ratio === "OneToTen") return "1:10";
      return "N/A";
    },
  },
  {
    name: "Puller Target Speed",
    unit: "m/min",
    renderValue: (data) => data.puller_state?.target_speed?.toFixed(2),
  },
  {
    name: "Puller Target Diameter",
    unit: "mm",
    renderValue: (data) => data.puller_state?.target_diameter?.toFixed(1),
  },
  previewSeparator,

  // ── Spool ──
  {
    name: "Spool Regulation",
    renderValue: (data) => data.spool_speed_controller_state?.regulation_mode,
  },
  {
    name: "Spool Direction",
    renderValue: (data) =>
      data.spool_speed_controller_state?.forward !== undefined
        ? data.spool_speed_controller_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Spool Min Speed",
    unit: "rpm",
    renderValue: (data) =>
      data.spool_speed_controller_state?.minmax_min_speed?.toFixed(2),
  },
  {
    name: "Spool Max Speed",
    unit: "rpm",
    renderValue: (data) =>
      data.spool_speed_controller_state?.minmax_max_speed?.toFixed(2),
  },
  previewSeparator,

  // ── Adaptive Spool ──
  {
    name: "Adaptive Tension Target",
    renderValue: (data) =>
      data.spool_speed_controller_state?.adaptive_tension_target?.toFixed(2),
  },
  {
    name: "Adaptive Learning Rate",
    renderValue: (data) =>
      data.spool_speed_controller_state?.adaptive_radius_learning_rate?.toFixed(
        2,
      ),
  },
  {
    name: "Adaptive Max Speed Multiplier",
    renderValue: (data) =>
      data.spool_speed_controller_state?.adaptive_max_speed_multiplier?.toFixed(
        1,
      ),
  },
  {
    name: "Adaptive Acceleration Factor",
    renderValue: (data) =>
      data.spool_speed_controller_state?.adaptive_acceleration_factor?.toFixed(
        2,
      ),
  },
  {
    name: "Adaptive Deaccel. Urgency",
    renderValue: (data) =>
      data.spool_speed_controller_state?.adaptive_deacceleration_urgency_multiplier?.toFixed(
        1,
      ),
  },
  previewSeparator,

  // ── Spool Automatic Action ──
  {
    name: "Spool Auto-Action Mode",
    renderValue: (data) =>
      data.spool_automatic_action_state?.spool_automatic_action_mode,
  },
  {
    name: "Spool Required Meters",
    unit: "m",
    renderValue: (data) =>
      data.spool_automatic_action_state?.spool_required_meters?.toFixed(1),
  },
  previewSeparator,

  // ── Heating ──
  {
    name: "Heating Enabled",
    renderValue: (data) =>
      data.heating_states?.enabled !== undefined
        ? data.heating_states.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Zone 1 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_1_target?.toFixed(0),
  },
  {
    name: "Zone 2 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_2_target?.toFixed(0),
  },
  {
    name: "Zone 3 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_3_target?.toFixed(0),
  },
  {
    name: "Zone 4 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_4_target?.toFixed(0),
  },
  {
    name: "Zone 5 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_5_target?.toFixed(0),
  },
  {
    name: "Zone 6 Temperature",
    unit: "C",
    renderValue: (data) => data.heating_states?.zone_6_target?.toFixed(0),
  },
  previewSeparator,

  // ── Addon Motor 3 ──
  {
    name: "Motor 3 Enabled",
    renderValue: (data) =>
      data.addon_motor_3_state?.enabled !== undefined
        ? data.addon_motor_3_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 3 Direction",
    renderValue: (data) =>
      data.addon_motor_3_state?.forward !== undefined
        ? data.addon_motor_3_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Motor 3 Master Ratio",
    renderValue: (data) => data.addon_motor_3_state?.master_ratio?.toFixed(2),
  },
  {
    name: "Motor 3 Slave Ratio",
    renderValue: (data) => data.addon_motor_3_state?.slave_ratio?.toFixed(2),
  },
  {
    name: "Motor 3 Konturlänge",
    unit: "mm",
    renderValue: (data) =>
      data.addon_motor_3_state?.konturlaenge_mm?.toFixed(1),
  },
  {
    name: "Motor 3 Pause",
    unit: "mm",
    renderValue: (data) => data.addon_motor_3_state?.pause_mm?.toFixed(1),
  },
  previewSeparator,

  // ── Addon Motor 4 ──
  {
    name: "Motor 4 Enabled",
    renderValue: (data) =>
      data.addon_motor_4_state?.enabled !== undefined
        ? data.addon_motor_4_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 4 Direction",
    renderValue: (data) =>
      data.addon_motor_4_state?.forward !== undefined
        ? data.addon_motor_4_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Motor 4 Master Ratio",
    renderValue: (data) => data.addon_motor_4_state?.master_ratio?.toFixed(2),
  },
  {
    name: "Motor 4 Slave Ratio",
    renderValue: (data) => data.addon_motor_4_state?.slave_ratio?.toFixed(2),
  },
  previewSeparator,

  // ── Addon Motor 5 ──
  {
    name: "Motor 5 Enabled",
    renderValue: (data) =>
      data.addon_motor_5_state?.enabled !== undefined
        ? data.addon_motor_5_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 5 Direction",
    renderValue: (data) =>
      data.addon_motor_5_state?.forward !== undefined
        ? data.addon_motor_5_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Motor 5 Master Ratio",
    renderValue: (data) => data.addon_motor_5_state?.master_ratio?.toFixed(2),
  },
  {
    name: "Motor 5 Slave Ratio",
    renderValue: (data) => data.addon_motor_5_state?.slave_ratio?.toFixed(2),
  },
  previewSeparator,

  // ── Slave Puller ──
  {
    name: "Slave Puller Enabled",
    renderValue: (data) =>
      data.slave_puller_state?.enabled !== undefined
        ? data.slave_puller_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Slave Puller Direction",
    renderValue: (data) =>
      data.slave_puller_state?.forward !== undefined
        ? data.slave_puller_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Slave Puller Target Angle",
    unit: "deg",
    renderValue: (data) => data.slave_puller_state?.target_angle?.toFixed(1),
  },
  {
    name: "Slave Puller Sensitivity",
    renderValue: (data) => data.slave_puller_state?.sensitivity?.toFixed(2),
  },
  {
    name: "Slave Puller Min Speed Factor",
    renderValue: (data) =>
      data.slave_puller_state?.min_speed_factor?.toFixed(2),
  },
  {
    name: "Slave Puller Max Speed Factor",
    renderValue: (data) =>
      data.slave_puller_state?.max_speed_factor?.toFixed(2),
  },
];

export function GluetexPresetsPage() {
  const {
    state,
    defaultState,

    // Traverse
    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    enableTraverseLaserpointer,

    // Puller
    setPullerRegulationMode,
    setPullerTargetSpeed,
    setPullerTargetDiameter,
    setPullerForward,
    setPullerGearRatio,

    // Spool
    setSpoolRegulationMode,
    setSpoolForward,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,

    // Spool Automatic Action
    setSpoolAutomaticRequiredMeters,
    setSpoolAutomaticAction,

    // Heating
    setHeatingMode,
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,

    // Addon Motors
    setStepper3Mode,
    setStepper3Forward,
    setStepper3Master,
    setStepper3Slave,
    setStepper3Konturlaenge,
    setStepper3Pause,
    setStepper4Mode,
    setStepper4Forward,
    setStepper4Master,
    setStepper4Slave,
    setStepper5Mode,
    setStepper5Forward,
    setStepper5Master,
    setStepper5Slave,

    // Slave Puller
    setSlavePullerEnabled,
    setSlavePullerForward,
    setSlavePullerTargetAngle,
    setSlavePullerSensitivity,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,
  } = useGluetex();

  const applyPreset = (preset: Preset<GluetexPresetData>) => {
    // Traverse
    setTraverseLimitInner(preset.data?.traverse_state?.limit_inner ?? 22);
    setTraverseLimitOuter(preset.data?.traverse_state?.limit_outer ?? 92);
    setTraverseStepSize(preset.data?.traverse_state?.step_size ?? 1.75);
    setTraversePadding(preset.data?.traverse_state?.padding ?? 0.88);
    enableTraverseLaserpointer(
      preset.data?.traverse_state?.laserpointer ?? false,
    );

    // Puller
    setPullerRegulationMode(preset.data?.puller_state?.regulation ?? "Speed");
    setPullerForward(preset.data?.puller_state?.forward ?? true);
    setPullerTargetSpeed(preset.data?.puller_state?.target_speed ?? 1.0);
    setPullerGearRatio(preset.data?.puller_state?.gear_ratio ?? "OneToOne");
    setPullerTargetDiameter(preset.data?.puller_state?.target_diameter ?? 1.75);

    // Spool
    setSpoolRegulationMode(
      preset.data?.spool_speed_controller_state?.regulation_mode ?? "MinMax",
    );
    setSpoolForward(preset.data?.spool_speed_controller_state?.forward ?? true);
    setSpoolMinMaxMinSpeed(
      preset.data?.spool_speed_controller_state?.minmax_min_speed ?? 0,
    );
    setSpoolMinMaxMaxSpeed(
      preset.data?.spool_speed_controller_state?.minmax_max_speed ?? 150.0,
    );
    setSpoolAdaptiveTensionTarget(
      preset.data?.spool_speed_controller_state?.adaptive_tension_target ?? 0.7,
    );
    setSpoolAdaptiveRadiusLearningRate(
      preset.data?.spool_speed_controller_state
        ?.adaptive_radius_learning_rate ?? 0.5,
    );
    setSpoolAdaptiveMaxSpeedMultiplier(
      preset.data?.spool_speed_controller_state
        ?.adaptive_max_speed_multiplier ?? 4,
    );
    setSpoolAdaptiveAccelerationFactor(
      preset.data?.spool_speed_controller_state?.adaptive_acceleration_factor ??
        0.2,
    );
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
      preset.data?.spool_speed_controller_state
        ?.adaptive_deacceleration_urgency_multiplier ?? 15.0,
    );

    // Spool Automatic Action
    setSpoolAutomaticRequiredMeters(
      preset.data?.spool_automatic_action_state?.spool_required_meters ?? 0,
    );
    setSpoolAutomaticAction(
      preset.data?.spool_automatic_action_state?.spool_automatic_action_mode ??
        "NoAction",
    );

    // Heating
    if (preset.data?.heating_states?.enabled !== undefined) {
      setHeatingMode(
        preset.data.heating_states.enabled ? "Heating" : "Standby",
      );
    }
    setHeatingZone1Temperature(preset.data?.heating_states?.zone_1_target ?? 0);
    setHeatingZone2Temperature(preset.data?.heating_states?.zone_2_target ?? 0);
    setHeatingZone3Temperature(preset.data?.heating_states?.zone_3_target ?? 0);
    setHeatingZone4Temperature(preset.data?.heating_states?.zone_4_target ?? 0);
    setHeatingZone5Temperature(preset.data?.heating_states?.zone_5_target ?? 0);
    setHeatingZone6Temperature(preset.data?.heating_states?.zone_6_target ?? 0);

    // Addon Motor 3
    if (preset.data?.addon_motor_3_state?.enabled !== undefined) {
      setStepper3Mode(
        preset.data.addon_motor_3_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_3_state?.forward !== undefined) {
      setStepper3Forward(preset.data.addon_motor_3_state.forward);
    }
    if (preset.data?.addon_motor_3_state?.master_ratio !== undefined) {
      setStepper3Master(preset.data.addon_motor_3_state.master_ratio);
    }
    if (preset.data?.addon_motor_3_state?.slave_ratio !== undefined) {
      setStepper3Slave(preset.data.addon_motor_3_state.slave_ratio);
    }
    if (preset.data?.addon_motor_3_state?.konturlaenge_mm !== undefined) {
      setStepper3Konturlaenge(preset.data.addon_motor_3_state.konturlaenge_mm);
    }
    if (preset.data?.addon_motor_3_state?.pause_mm !== undefined) {
      setStepper3Pause(preset.data.addon_motor_3_state.pause_mm);
    }

    // Addon Motor 4
    if (preset.data?.addon_motor_4_state?.enabled !== undefined) {
      setStepper4Mode(
        preset.data.addon_motor_4_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_4_state?.forward !== undefined) {
      setStepper4Forward(preset.data.addon_motor_4_state.forward);
    }
    if (preset.data?.addon_motor_4_state?.master_ratio !== undefined) {
      setStepper4Master(preset.data.addon_motor_4_state.master_ratio);
    }
    if (preset.data?.addon_motor_4_state?.slave_ratio !== undefined) {
      setStepper4Slave(preset.data.addon_motor_4_state.slave_ratio);
    }

    // Addon Motor 5
    if (preset.data?.addon_motor_5_state?.enabled !== undefined) {
      setStepper5Mode(
        preset.data.addon_motor_5_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_5_state?.forward !== undefined) {
      setStepper5Forward(preset.data.addon_motor_5_state.forward);
    }
    if (preset.data?.addon_motor_5_state?.master_ratio !== undefined) {
      setStepper5Master(preset.data.addon_motor_5_state.master_ratio);
    }
    if (preset.data?.addon_motor_5_state?.slave_ratio !== undefined) {
      setStepper5Slave(preset.data.addon_motor_5_state.slave_ratio);
    }

    // Slave Puller
    if (preset.data?.slave_puller_state?.enabled !== undefined) {
      setSlavePullerEnabled(preset.data.slave_puller_state.enabled);
    }
    if (preset.data?.slave_puller_state?.forward !== undefined) {
      setSlavePullerForward(preset.data.slave_puller_state.forward);
    }
    if (preset.data?.slave_puller_state?.target_angle !== undefined) {
      setSlavePullerTargetAngle(preset.data.slave_puller_state.target_angle);
    }
    if (preset.data?.slave_puller_state?.sensitivity !== undefined) {
      setSlavePullerSensitivity(preset.data.slave_puller_state.sensitivity);
    }
    if (preset.data?.slave_puller_state?.min_speed_factor !== undefined) {
      setSlavePullerMinSpeedFactor(
        preset.data.slave_puller_state.min_speed_factor,
      );
    }
    if (preset.data?.slave_puller_state?.max_speed_factor !== undefined) {
      setSlavePullerMaxSpeedFactor(
        preset.data.slave_puller_state.max_speed_factor,
      );
    }
  };

  const toPresetData = (s: typeof state): GluetexPresetData => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state ?? {},
    spool_speed_controller_state: s?.spool_speed_controller_state ?? {},
    spool_automatic_action_state: s?.spool_automatic_action_state ?? {},
    heating_states: {
      enabled: s?.heating_states?.enabled,
      zone_1_target: s?.heating_states?.zone_1?.target_temperature,
      zone_2_target: s?.heating_states?.zone_2?.target_temperature,
      zone_3_target: s?.heating_states?.zone_3?.target_temperature,
      zone_4_target: s?.heating_states?.zone_4?.target_temperature,
      zone_5_target: s?.heating_states?.zone_5?.target_temperature,
      zone_6_target: s?.heating_states?.zone_6?.target_temperature,
    },
    addon_motor_3_state: s?.addon_motor_3_state
      ? {
          enabled: s.addon_motor_3_state.enabled,
          forward: s.addon_motor_3_state.forward,
          master_ratio: s.addon_motor_3_state.master_ratio,
          slave_ratio: s.addon_motor_3_state.slave_ratio,
          konturlaenge_mm: s.addon_motor_3_state.konturlaenge_mm,
          pause_mm: s.addon_motor_3_state.pause_mm,
          pattern_state: s.addon_motor_3_state.pattern_state,
        }
      : {},
    addon_motor_4_state: s?.addon_motor_4_state
      ? {
          enabled: s.addon_motor_4_state.enabled,
          forward: s.addon_motor_4_state.forward,
          master_ratio: s.addon_motor_4_state.master_ratio,
          slave_ratio: s.addon_motor_4_state.slave_ratio,
        }
      : {},
    addon_motor_5_state: s?.addon_motor_5_state
      ? {
          enabled: s.addon_motor_5_state.enabled,
          forward: s.addon_motor_5_state.forward,
          master_ratio: s.addon_motor_5_state.master_ratio,
          slave_ratio: s.addon_motor_5_state.slave_ratio,
        }
      : {},
    slave_puller_state: s?.slave_puller_state
      ? {
          enabled: s.slave_puller_state.enabled,
          forward: s.slave_puller_state.forward,
          target_angle: s.slave_puller_state.target_angle,
          sensitivity: s.slave_puller_state.sensitivity,
          min_speed_factor: s.slave_puller_state.min_speed_factor ?? undefined,
          max_speed_factor: s.slave_puller_state.max_speed_factor ?? undefined,
        }
      : {},
  });

  return (
    <PresetsPage
      machine_identification={gluetex.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      defaultState={toPresetData(defaultState)}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
    />
  );
}
