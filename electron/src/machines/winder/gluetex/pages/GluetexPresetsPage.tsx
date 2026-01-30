import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import { gluetex } from "@/machines/properties";
import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import {
  pullerStateSchema,
  spoolSpeedControllerStateSchema,
  heatingStatesSchema,
  heatingPidStatesSchema,
  addonMotorStateSchema,
  addonMotor5StateSchema,
  slavePullerStateSchema,
  tensionArmMonitorStateSchema,
} from "../state/gluetexNamespace";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";

// Define the preset data schema for Gluetex
// Including: winder (traverse, puller, spool), addons (motors 3,4,5, slave puller, tension arm monitor), temp (heating), and settings (quality control)
const gluetexPresetDataSchema = z
  .object({
    // Winder settings - Traverse
    traverse_state: z
      .object({
        limit_inner: z.number(),
        limit_outer: z.number(),
        step_size: z.number(),
        padding: z.number(),
        laserpointer: z.boolean(),
      })
      .partial(),

    // Winder settings - Puller
    puller_state: pullerStateSchema.partial(),

    // Winder settings - Spool speed controller
    spool_speed_controller_state: spoolSpeedControllerStateSchema.partial(),

    // Temperature/Heating settings
    heating_states: heatingStatesSchema.partial(),
    heating_pid_settings: heatingPidStatesSchema.partial(),

    // Addon settings - Motors
    addon_motor_3_state: addonMotorStateSchema.partial(),
    addon_motor_4_state: addonMotorStateSchema.partial(),
    addon_motor_5_state: addonMotor5StateSchema.partial(),

    // Addon settings - Slave puller
    slave_puller_state: slavePullerStateSchema.partial(),

    // Addon settings - Tension arm monitor
    tension_arm_monitor_state: tensionArmMonitorStateSchema.partial(),

    // Settings - Quality control (local state)
    quality_control_state: z
      .object({
        temperature1: z
          .object({
            min_temperature: z.number(),
            max_temperature: z.number(),
          })
          .partial(),
        temperature2: z
          .object({
            min_temperature: z.number(),
            max_temperature: z.number(),
          })
          .partial(),
      })
      .partial(),
  })
  .partial();

type GluetexPreset = z.infer<typeof gluetexPresetDataSchema>;

const schemas = new Map([[1, gluetexPresetDataSchema]]);

// Preview entries for the preset cards
const previewEntries: PresetPreviewEntries<GluetexPreset> = [
  // Winder - Traverse
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.limit_inner?.toFixed(1) ?? "N/A",
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.limit_outer?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Step Size",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.step_size?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.padding?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Winder - Puller
  {
    name: "Puller Regulation",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.regulation ?? "N/A",
  },
  {
    name: "Puller Direction",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.forward !== undefined
        ? data.puller_state.forward
          ? "Forward"
          : "Backward"
        : "N/A",
  },
  {
    name: "Puller Gear Ratio",
    renderValue: (data: GluetexPreset) => {
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
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.target_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Puller Target Diameter",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.target_diameter?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Winder - Spool
  {
    name: "Spool Regulation",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.regulation_mode ?? "N/A",
  },
  {
    name: "Spool Direction",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.forward !== undefined
        ? data.spool_speed_controller_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Spool Min Speed",
    unit: "rpm",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.minmax_min_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Spool Max Speed",
    unit: "rpm",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.minmax_max_speed?.toFixed(2) ?? "N/A",
  },
  previewSeparator,
  // Temperature - Heating zones
  {
    name: "Heating Enabled",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.enabled !== undefined
        ? data.heating_states.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Zone 1 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_1?.target_temperature?.toFixed(1) ?? "N/A",
  },
  {
    name: "Zone 2 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_2?.target_temperature?.toFixed(1) ?? "N/A",
  },
  {
    name: "Zone 3 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_3?.target_temperature?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Addons - Motors
  {
    name: "Motor 3 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_3_state?.enabled !== undefined
        ? data.addon_motor_3_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 3 Master Ratio",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_3_state?.master_ratio?.toFixed(2) ?? "N/A",
  },
  {
    name: "Motor 4 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_4_state?.enabled !== undefined
        ? data.addon_motor_4_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 5 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_5_state?.enabled !== undefined
        ? data.addon_motor_5_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  previewSeparator,
  // Addons - Slave puller
  {
    name: "Slave Puller Enabled",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.enabled !== undefined
        ? data.slave_puller_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Slave Puller Min Angle",
    unit: "deg",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.min_angle?.toFixed(1) ?? "N/A",
  },
  {
    name: "Slave Puller Max Angle",
    unit: "deg",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.max_angle?.toFixed(1) ?? "N/A",
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
    setPullerTargetDiameter,
    setPullerForward,
    setPullerTargetSpeed,
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

    // Heating/Temperature
    setHeatingMode,
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,
    setHeatingPid,

    // Addons - Motors
    setStepper3Mode,
    setStepper3Master,
    setStepper3Slave,
    setStepper4Mode,
    setStepper4Master,
    setStepper4Slave,
    setStepper5Mode,
    setStepper5Master,
    setStepper5Slave,
    setStepper5Konturlaenge,
    setStepper5Pause,

    // Addons - Slave puller
    setSlavePullerEnabled,
    setSlavePullerForward,
    setSlavePullerMinAngle,
    setSlavePullerMaxAngle,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,

    // Addons - Tension arm monitor
    setTensionArmMonitorEnabled,
    setTensionArmMonitorMinAngle,
    setTensionArmMonitorMaxAngle,

    // Settings - Quality control
    setTemperature1Min,
    setTemperature1Max,
    setTemperature2Min,
    setTemperature2Max,
  } = useGluetex();

  const applyPreset = (preset: Preset<GluetexPreset>) => {
    // Apply traverse settings
    if (preset.data?.traverse_state?.limit_inner !== undefined) {
      setTraverseLimitInner(preset.data.traverse_state.limit_inner);
    }
    if (preset.data?.traverse_state?.limit_outer !== undefined) {
      setTraverseLimitOuter(preset.data.traverse_state.limit_outer);
    }
    if (preset.data?.traverse_state?.step_size !== undefined) {
      setTraverseStepSize(preset.data.traverse_state.step_size);
    }
    if (preset.data?.traverse_state?.padding !== undefined) {
      setTraversePadding(preset.data.traverse_state.padding);
    }
    if (preset.data?.traverse_state?.laserpointer !== undefined) {
      enableTraverseLaserpointer(preset.data.traverse_state.laserpointer);
    }

    // Apply puller settings
    if (preset.data?.puller_state?.regulation !== undefined) {
      setPullerRegulationMode(preset.data.puller_state.regulation);
    }
    if (preset.data?.puller_state?.forward !== undefined) {
      setPullerForward(preset.data.puller_state.forward);
    }
    if (preset.data?.puller_state?.target_speed !== undefined) {
      setPullerTargetSpeed(preset.data.puller_state.target_speed);
    }
    if (preset.data?.puller_state?.gear_ratio !== undefined) {
      setPullerGearRatio(preset.data.puller_state.gear_ratio);
    }
    if (preset.data?.puller_state?.target_diameter !== undefined) {
      setPullerTargetDiameter(preset.data.puller_state.target_diameter);
    }

    // Apply spool settings
    if (
      preset.data?.spool_speed_controller_state?.regulation_mode !== undefined
    ) {
      setSpoolRegulationMode(
        preset.data.spool_speed_controller_state.regulation_mode,
      );
    }
    if (preset.data?.spool_speed_controller_state?.forward !== undefined) {
      setSpoolForward(preset.data.spool_speed_controller_state.forward);
    }
    if (
      preset.data?.spool_speed_controller_state?.minmax_min_speed !== undefined
    ) {
      setSpoolMinMaxMinSpeed(
        preset.data.spool_speed_controller_state.minmax_min_speed,
      );
    }
    if (
      preset.data?.spool_speed_controller_state?.minmax_max_speed !== undefined
    ) {
      setSpoolMinMaxMaxSpeed(
        preset.data.spool_speed_controller_state.minmax_max_speed,
      );
    }
    if (
      preset.data?.spool_speed_controller_state?.adaptive_tension_target !==
      undefined
    ) {
      setSpoolAdaptiveTensionTarget(
        preset.data.spool_speed_controller_state.adaptive_tension_target,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_radius_learning_rate !== undefined
    ) {
      setSpoolAdaptiveRadiusLearningRate(
        preset.data.spool_speed_controller_state.adaptive_radius_learning_rate,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_max_speed_multiplier !== undefined
    ) {
      setSpoolAdaptiveMaxSpeedMultiplier(
        preset.data.spool_speed_controller_state.adaptive_max_speed_multiplier,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_acceleration_factor !== undefined
    ) {
      setSpoolAdaptiveAccelerationFactor(
        preset.data.spool_speed_controller_state.adaptive_acceleration_factor,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_deacceleration_urgency_multiplier !== undefined
    ) {
      setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
        preset.data.spool_speed_controller_state
          .adaptive_deacceleration_urgency_multiplier,
      );
    }

    // Apply heating settings
    if (preset.data?.heating_states?.enabled !== undefined) {
      setHeatingMode(
        preset.data.heating_states.enabled ? "Heating" : "Standby",
      );
    }
    if (preset.data?.heating_states?.zone_1?.target_temperature !== undefined) {
      setHeatingZone1Temperature(
        preset.data.heating_states.zone_1.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_2?.target_temperature !== undefined) {
      setHeatingZone2Temperature(
        preset.data.heating_states.zone_2.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_3?.target_temperature !== undefined) {
      setHeatingZone3Temperature(
        preset.data.heating_states.zone_3.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_4?.target_temperature !== undefined) {
      setHeatingZone4Temperature(
        preset.data.heating_states.zone_4.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_5?.target_temperature !== undefined) {
      setHeatingZone5Temperature(
        preset.data.heating_states.zone_5.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_6?.target_temperature !== undefined) {
      setHeatingZone6Temperature(
        preset.data.heating_states.zone_6.target_temperature,
      );
    }

    // Apply heating PID settings
    if (preset.data?.heating_pid_settings?.zone_1) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_1;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_1", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_2) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_2;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_2", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_3) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_3;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_3", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_4) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_4;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_4", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_5) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_5;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_5", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_6) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_6;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_6", kp, ki, kd);
      }
    }

    // Apply addon motor 3 settings
    if (preset.data?.addon_motor_3_state?.enabled !== undefined) {
      setStepper3Mode(
        preset.data.addon_motor_3_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_3_state?.master_ratio !== undefined) {
      setStepper3Master(preset.data.addon_motor_3_state.master_ratio);
    }
    if (preset.data?.addon_motor_3_state?.slave_ratio !== undefined) {
      setStepper3Slave(preset.data.addon_motor_3_state.slave_ratio);
    }

    // Apply addon motor 4 settings
    if (preset.data?.addon_motor_4_state?.enabled !== undefined) {
      setStepper4Mode(
        preset.data.addon_motor_4_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_4_state?.master_ratio !== undefined) {
      setStepper4Master(preset.data.addon_motor_4_state.master_ratio);
    }
    if (preset.data?.addon_motor_4_state?.slave_ratio !== undefined) {
      setStepper4Slave(preset.data.addon_motor_4_state.slave_ratio);
    }

    // Apply addon motor 5 settings
    if (preset.data?.addon_motor_5_state?.enabled !== undefined) {
      setStepper5Mode(
        preset.data.addon_motor_5_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_5_state?.master_ratio !== undefined) {
      setStepper5Master(preset.data.addon_motor_5_state.master_ratio);
    }
    if (preset.data?.addon_motor_5_state?.slave_ratio !== undefined) {
      setStepper5Slave(preset.data.addon_motor_5_state.slave_ratio);
    }
    if (preset.data?.addon_motor_5_state?.konturlaenge_mm !== undefined) {
      setStepper5Konturlaenge(preset.data.addon_motor_5_state.konturlaenge_mm);
    }
    if (preset.data?.addon_motor_5_state?.pause_mm !== undefined) {
      setStepper5Pause(preset.data.addon_motor_5_state.pause_mm);
    }

    // Apply slave puller settings
    if (preset.data?.slave_puller_state?.enabled !== undefined) {
      setSlavePullerEnabled(preset.data.slave_puller_state.enabled);
    }
    if (preset.data?.slave_puller_state?.forward !== undefined) {
      setSlavePullerForward(preset.data.slave_puller_state.forward);
    }
    if (preset.data?.slave_puller_state?.min_angle !== undefined) {
      setSlavePullerMinAngle(preset.data.slave_puller_state.min_angle);
    }
    if (preset.data?.slave_puller_state?.max_angle !== undefined) {
      setSlavePullerMaxAngle(preset.data.slave_puller_state.max_angle);
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

    // Apply tension arm monitor settings
    if (preset.data?.tension_arm_monitor_state?.enabled !== undefined) {
      setTensionArmMonitorEnabled(
        preset.data.tension_arm_monitor_state.enabled,
      );
    }
    if (preset.data?.tension_arm_monitor_state?.min_angle !== undefined) {
      setTensionArmMonitorMinAngle(
        preset.data.tension_arm_monitor_state.min_angle,
      );
    }
    if (preset.data?.tension_arm_monitor_state?.max_angle !== undefined) {
      setTensionArmMonitorMaxAngle(
        preset.data.tension_arm_monitor_state.max_angle,
      );
    }

    // Apply quality control settings
    if (
      preset.data?.quality_control_state?.temperature1?.min_temperature !==
      undefined
    ) {
      setTemperature1Min(
        preset.data.quality_control_state.temperature1.min_temperature,
      );
    }
    if (
      preset.data?.quality_control_state?.temperature1?.max_temperature !==
      undefined
    ) {
      setTemperature1Max(
        preset.data.quality_control_state.temperature1.max_temperature,
      );
    }
    if (
      preset.data?.quality_control_state?.temperature2?.min_temperature !==
      undefined
    ) {
      setTemperature2Min(
        preset.data.quality_control_state.temperature2.min_temperature,
      );
    }
    if (
      preset.data?.quality_control_state?.temperature2?.max_temperature !==
      undefined
    ) {
      setTemperature2Max(
        preset.data.quality_control_state.temperature2.max_temperature,
      );
    }
  };

  const toPresetData = (s: typeof state): GluetexPreset => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state,
    spool_speed_controller_state: s?.spool_speed_controller_state,
    heating_states: s?.heating_states,
    heating_pid_settings: s?.heating_pid_settings,
    addon_motor_3_state: s?.addon_motor_3_state,
    addon_motor_4_state: s?.addon_motor_4_state,
    addon_motor_5_state: s?.addon_motor_5_state,
    slave_puller_state: s?.slave_puller_state,
    tension_arm_monitor_state: s?.tension_arm_monitor_state,
    quality_control_state: s?.quality_control_state,
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
