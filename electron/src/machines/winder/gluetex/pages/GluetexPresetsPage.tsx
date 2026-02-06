import React from "react";
import { toast } from "sonner";
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
  heatingPidSettingsSchema,
  tensionArmMonitorStateSchema,
  voltageMonitorStateSchema,
  sleepTimerStateSchema,
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
    heating_pid_settings: z
      .object({
        zone_1: heatingPidSettingsSchema.omit({ zone: true }).partial(),
        zone_2: heatingPidSettingsSchema.omit({ zone: true }).partial(),
        zone_3: heatingPidSettingsSchema.omit({ zone: true }).partial(),
        zone_4: heatingPidSettingsSchema.omit({ zone: true }).partial(),
        zone_5: heatingPidSettingsSchema.omit({ zone: true }).partial(),
        zone_6: heatingPidSettingsSchema.omit({ zone: true }).partial(),
      })
      .partial(),
    winder_tension_arm_monitor_state: tensionArmMonitorStateSchema
      .omit({ triggered: true })
      .partial(),
    addon_tension_arm_monitor_state: tensionArmMonitorStateSchema
      .omit({ triggered: true })
      .partial(),
    slave_tension_arm_monitor_state: tensionArmMonitorStateSchema
      .omit({ triggered: true })
      .partial(),
    optris_1_monitor_state: voltageMonitorStateSchema
      .omit({ triggered: true })
      .partial(),
    optris_2_monitor_state: voltageMonitorStateSchema
      .omit({ triggered: true })
      .partial(),
    sleep_timer_state: sleepTimerStateSchema
      .pick({ enabled: true, timeout_seconds: true })
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
  previewSeparator,

  // ── Heating PID ──
  {
    name: "Zone 1 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_1?.kp?.toFixed(3),
  },
  {
    name: "Zone 1 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_1?.ki?.toFixed(3),
  },
  {
    name: "Zone 1 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_1?.kd?.toFixed(3),
  },
  {
    name: "Zone 2 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_2?.kp?.toFixed(3),
  },
  {
    name: "Zone 2 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_2?.ki?.toFixed(3),
  },
  {
    name: "Zone 2 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_2?.kd?.toFixed(3),
  },
  {
    name: "Zone 3 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_3?.kp?.toFixed(3),
  },
  {
    name: "Zone 3 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_3?.ki?.toFixed(3),
  },
  {
    name: "Zone 3 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_3?.kd?.toFixed(3),
  },
  {
    name: "Zone 4 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_4?.kp?.toFixed(3),
  },
  {
    name: "Zone 4 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_4?.ki?.toFixed(3),
  },
  {
    name: "Zone 4 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_4?.kd?.toFixed(3),
  },
  {
    name: "Zone 5 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_5?.kp?.toFixed(3),
  },
  {
    name: "Zone 5 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_5?.ki?.toFixed(3),
  },
  {
    name: "Zone 5 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_5?.kd?.toFixed(3),
  },
  {
    name: "Zone 6 PID Kp",
    renderValue: (data) => data.heating_pid_settings?.zone_6?.kp?.toFixed(3),
  },
  {
    name: "Zone 6 PID Ki",
    renderValue: (data) => data.heating_pid_settings?.zone_6?.ki?.toFixed(3),
  },
  {
    name: "Zone 6 PID Kd",
    renderValue: (data) => data.heating_pid_settings?.zone_6?.kd?.toFixed(3),
  },
  previewSeparator,

  // ── Winder Tension Arm Monitor ──
  {
    name: "Winder Tension Monitor",
    renderValue: (data) =>
      data.winder_tension_arm_monitor_state?.enabled !== undefined
        ? data.winder_tension_arm_monitor_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Winder Tension Min Angle",
    unit: "deg",
    renderValue: (data) =>
      data.winder_tension_arm_monitor_state?.min_angle?.toFixed(1),
  },
  {
    name: "Winder Tension Max Angle",
    unit: "deg",
    renderValue: (data) =>
      data.winder_tension_arm_monitor_state?.max_angle?.toFixed(1),
  },
  previewSeparator,

  // ── Addon Tension Arm Monitor ──
  {
    name: "Addon Tension Monitor",
    renderValue: (data) =>
      data.addon_tension_arm_monitor_state?.enabled !== undefined
        ? data.addon_tension_arm_monitor_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Addon Tension Min Angle",
    unit: "deg",
    renderValue: (data) =>
      data.addon_tension_arm_monitor_state?.min_angle?.toFixed(1),
  },
  {
    name: "Addon Tension Max Angle",
    unit: "deg",
    renderValue: (data) =>
      data.addon_tension_arm_monitor_state?.max_angle?.toFixed(1),
  },
  previewSeparator,

  // ── Slave Tension Arm Monitor ──
  {
    name: "Slave Tension Monitor",
    renderValue: (data) =>
      data.slave_tension_arm_monitor_state?.enabled !== undefined
        ? data.slave_tension_arm_monitor_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Slave Tension Min Angle",
    unit: "deg",
    renderValue: (data) =>
      data.slave_tension_arm_monitor_state?.min_angle?.toFixed(1),
  },
  {
    name: "Slave Tension Max Angle",
    unit: "deg",
    renderValue: (data) =>
      data.slave_tension_arm_monitor_state?.max_angle?.toFixed(1),
  },
  previewSeparator,

  // ── Optris 1 Monitor ──
  {
    name: "Optris 1 Monitor",
    renderValue: (data) =>
      data.optris_1_monitor_state?.enabled !== undefined
        ? data.optris_1_monitor_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Optris 1 Min Voltage",
    unit: "V",
    renderValue: (data) => data.optris_1_monitor_state?.min_voltage?.toFixed(2),
  },
  {
    name: "Optris 1 Max Voltage",
    unit: "V",
    renderValue: (data) => data.optris_1_monitor_state?.max_voltage?.toFixed(2),
  },
  previewSeparator,

  // ── Optris 2 Monitor ──
  {
    name: "Optris 2 Monitor",
    renderValue: (data) =>
      data.optris_2_monitor_state?.enabled !== undefined
        ? data.optris_2_monitor_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Optris 2 Min Voltage",
    unit: "V",
    renderValue: (data) => data.optris_2_monitor_state?.min_voltage?.toFixed(2),
  },
  {
    name: "Optris 2 Max Voltage",
    unit: "V",
    renderValue: (data) => data.optris_2_monitor_state?.max_voltage?.toFixed(2),
  },
  previewSeparator,

  // ── Sleep Timer ──
  {
    name: "Sleep Timer",
    renderValue: (data) =>
      data.sleep_timer_state?.enabled !== undefined
        ? data.sleep_timer_state.enabled
          ? "Enabled"
          : "Disabled"
        : "N/A",
  },
  {
    name: "Sleep Timer Timeout",
    unit: "s",
    renderValue: (data) => data.sleep_timer_state?.timeout_seconds?.toFixed(0),
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

    // Heating PID
    setHeatingPid,

    // Tension Arm Monitors
    setWinderTensionArmMonitorEnabled,
    setWinderTensionArmMonitorMinAngle,
    setWinderTensionArmMonitorMaxAngle,
    setAddonTensionArmMonitorEnabled,
    setAddonTensionArmMonitorMinAngle,
    setAddonTensionArmMonitorMaxAngle,
    setSlaveTensionArmMonitorEnabled,
    setSlaveTensionArmMonitorMinAngle,
    setSlaveTensionArmMonitorMaxAngle,

    // Optris Monitors
    setOptris1MonitorEnabled,
    setOptris1MonitorMinVoltage,
    setOptris1MonitorMaxVoltage,
    setOptris2MonitorEnabled,
    setOptris2MonitorMinVoltage,
    setOptris2MonitorMaxVoltage,

    // Sleep Timer
    setSleepTimerEnabled,
    setSleepTimerTimeout,
  } = useGluetex();

  const applyPreset = async (preset: Preset<GluetexPresetData>) => {
    const currentData = toPresetData(state);
    const d = preset.data;

    // Build a list of setter calls for values that differ from current state
    const actions: Array<() => void> = [];

    const changed = <T,>(
      presetVal: T | undefined,
      currentVal: T | undefined,
    ): presetVal is T => {
      if (presetVal === undefined) return false;
      return presetVal !== currentVal;
    };

    // Traverse
    if (
      changed(
        d?.traverse_state?.limit_inner,
        currentData.traverse_state?.limit_inner,
      )
    )
      actions.push(() => setTraverseLimitInner(d.traverse_state!.limit_inner!));
    if (
      changed(
        d?.traverse_state?.limit_outer,
        currentData.traverse_state?.limit_outer,
      )
    )
      actions.push(() => setTraverseLimitOuter(d.traverse_state!.limit_outer!));
    if (
      changed(
        d?.traverse_state?.step_size,
        currentData.traverse_state?.step_size,
      )
    )
      actions.push(() => setTraverseStepSize(d.traverse_state!.step_size!));
    if (
      changed(d?.traverse_state?.padding, currentData.traverse_state?.padding)
    )
      actions.push(() => setTraversePadding(d.traverse_state!.padding!));
    if (
      changed(
        d?.traverse_state?.laserpointer,
        currentData.traverse_state?.laserpointer,
      )
    )
      actions.push(() =>
        enableTraverseLaserpointer(d.traverse_state!.laserpointer!),
      );

    // Puller
    if (
      changed(d?.puller_state?.regulation, currentData.puller_state?.regulation)
    )
      actions.push(() => setPullerRegulationMode(d.puller_state!.regulation!));
    if (changed(d?.puller_state?.forward, currentData.puller_state?.forward))
      actions.push(() => setPullerForward(d.puller_state!.forward!));
    if (
      changed(
        d?.puller_state?.target_speed,
        currentData.puller_state?.target_speed,
      )
    )
      actions.push(() => setPullerTargetSpeed(d.puller_state!.target_speed!));
    if (
      changed(d?.puller_state?.gear_ratio, currentData.puller_state?.gear_ratio)
    )
      actions.push(() => setPullerGearRatio(d.puller_state!.gear_ratio!));
    if (
      changed(
        d?.puller_state?.target_diameter,
        currentData.puller_state?.target_diameter,
      )
    )
      actions.push(() =>
        setPullerTargetDiameter(d.puller_state!.target_diameter!),
      );

    // Spool
    if (
      changed(
        d?.spool_speed_controller_state?.regulation_mode,
        currentData.spool_speed_controller_state?.regulation_mode,
      )
    )
      actions.push(() =>
        setSpoolRegulationMode(
          d.spool_speed_controller_state!.regulation_mode!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.forward,
        currentData.spool_speed_controller_state?.forward,
      )
    )
      actions.push(() =>
        setSpoolForward(d.spool_speed_controller_state!.forward!),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.minmax_min_speed,
        currentData.spool_speed_controller_state?.minmax_min_speed,
      )
    )
      actions.push(() =>
        setSpoolMinMaxMinSpeed(
          d.spool_speed_controller_state!.minmax_min_speed!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.minmax_max_speed,
        currentData.spool_speed_controller_state?.minmax_max_speed,
      )
    )
      actions.push(() =>
        setSpoolMinMaxMaxSpeed(
          d.spool_speed_controller_state!.minmax_max_speed!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.adaptive_tension_target,
        currentData.spool_speed_controller_state?.adaptive_tension_target,
      )
    )
      actions.push(() =>
        setSpoolAdaptiveTensionTarget(
          d.spool_speed_controller_state!.adaptive_tension_target!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.adaptive_radius_learning_rate,
        currentData.spool_speed_controller_state?.adaptive_radius_learning_rate,
      )
    )
      actions.push(() =>
        setSpoolAdaptiveRadiusLearningRate(
          d.spool_speed_controller_state!.adaptive_radius_learning_rate!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.adaptive_max_speed_multiplier,
        currentData.spool_speed_controller_state?.adaptive_max_speed_multiplier,
      )
    )
      actions.push(() =>
        setSpoolAdaptiveMaxSpeedMultiplier(
          d.spool_speed_controller_state!.adaptive_max_speed_multiplier!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state?.adaptive_acceleration_factor,
        currentData.spool_speed_controller_state?.adaptive_acceleration_factor,
      )
    )
      actions.push(() =>
        setSpoolAdaptiveAccelerationFactor(
          d.spool_speed_controller_state!.adaptive_acceleration_factor!,
        ),
      );
    if (
      changed(
        d?.spool_speed_controller_state
          ?.adaptive_deacceleration_urgency_multiplier,
        currentData.spool_speed_controller_state
          ?.adaptive_deacceleration_urgency_multiplier,
      )
    )
      actions.push(() =>
        setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
          d.spool_speed_controller_state!
            .adaptive_deacceleration_urgency_multiplier!,
        ),
      );

    // Spool Automatic Action
    if (
      changed(
        d?.spool_automatic_action_state?.spool_required_meters,
        currentData.spool_automatic_action_state?.spool_required_meters,
      )
    )
      actions.push(() =>
        setSpoolAutomaticRequiredMeters(
          d.spool_automatic_action_state!.spool_required_meters!,
        ),
      );
    if (
      changed(
        d?.spool_automatic_action_state?.spool_automatic_action_mode,
        currentData.spool_automatic_action_state?.spool_automatic_action_mode,
      )
    )
      actions.push(() =>
        setSpoolAutomaticAction(
          d.spool_automatic_action_state!.spool_automatic_action_mode!,
        ),
      );

    // Heating
    if (
      d?.heating_states?.enabled !== undefined &&
      d.heating_states.enabled !== currentData.heating_states?.enabled
    )
      actions.push(() =>
        setHeatingMode(d.heating_states!.enabled ? "Heating" : "Standby"),
      );
    if (
      changed(
        d?.heating_states?.zone_1_target,
        currentData.heating_states?.zone_1_target,
      )
    )
      actions.push(() =>
        setHeatingZone1Temperature(d.heating_states!.zone_1_target!),
      );
    if (
      changed(
        d?.heating_states?.zone_2_target,
        currentData.heating_states?.zone_2_target,
      )
    )
      actions.push(() =>
        setHeatingZone2Temperature(d.heating_states!.zone_2_target!),
      );
    if (
      changed(
        d?.heating_states?.zone_3_target,
        currentData.heating_states?.zone_3_target,
      )
    )
      actions.push(() =>
        setHeatingZone3Temperature(d.heating_states!.zone_3_target!),
      );
    if (
      changed(
        d?.heating_states?.zone_4_target,
        currentData.heating_states?.zone_4_target,
      )
    )
      actions.push(() =>
        setHeatingZone4Temperature(d.heating_states!.zone_4_target!),
      );
    if (
      changed(
        d?.heating_states?.zone_5_target,
        currentData.heating_states?.zone_5_target,
      )
    )
      actions.push(() =>
        setHeatingZone5Temperature(d.heating_states!.zone_5_target!),
      );
    if (
      changed(
        d?.heating_states?.zone_6_target,
        currentData.heating_states?.zone_6_target,
      )
    )
      actions.push(() =>
        setHeatingZone6Temperature(d.heating_states!.zone_6_target!),
      );

    // Addon Motor 3
    if (
      d?.addon_motor_3_state?.enabled !== undefined &&
      d.addon_motor_3_state.enabled !== currentData.addon_motor_3_state?.enabled
    )
      actions.push(() =>
        setStepper3Mode(d.addon_motor_3_state!.enabled ? "Run" : "Standby"),
      );
    if (
      changed(
        d?.addon_motor_3_state?.forward,
        currentData.addon_motor_3_state?.forward,
      )
    )
      actions.push(() => setStepper3Forward(d.addon_motor_3_state!.forward!));
    if (
      changed(
        d?.addon_motor_3_state?.master_ratio,
        currentData.addon_motor_3_state?.master_ratio,
      )
    )
      actions.push(() =>
        setStepper3Master(d.addon_motor_3_state!.master_ratio!),
      );
    if (
      changed(
        d?.addon_motor_3_state?.slave_ratio,
        currentData.addon_motor_3_state?.slave_ratio,
      )
    )
      actions.push(() => setStepper3Slave(d.addon_motor_3_state!.slave_ratio!));
    if (
      changed(
        d?.addon_motor_3_state?.konturlaenge_mm,
        currentData.addon_motor_3_state?.konturlaenge_mm,
      )
    )
      actions.push(() =>
        setStepper3Konturlaenge(d.addon_motor_3_state!.konturlaenge_mm!),
      );
    if (
      changed(
        d?.addon_motor_3_state?.pause_mm,
        currentData.addon_motor_3_state?.pause_mm,
      )
    )
      actions.push(() => setStepper3Pause(d.addon_motor_3_state!.pause_mm!));

    // Addon Motor 4
    if (
      d?.addon_motor_4_state?.enabled !== undefined &&
      d.addon_motor_4_state.enabled !== currentData.addon_motor_4_state?.enabled
    )
      actions.push(() =>
        setStepper4Mode(d.addon_motor_4_state!.enabled ? "Run" : "Standby"),
      );
    if (
      changed(
        d?.addon_motor_4_state?.forward,
        currentData.addon_motor_4_state?.forward,
      )
    )
      actions.push(() => setStepper4Forward(d.addon_motor_4_state!.forward!));
    if (
      changed(
        d?.addon_motor_4_state?.master_ratio,
        currentData.addon_motor_4_state?.master_ratio,
      )
    )
      actions.push(() =>
        setStepper4Master(d.addon_motor_4_state!.master_ratio!),
      );
    if (
      changed(
        d?.addon_motor_4_state?.slave_ratio,
        currentData.addon_motor_4_state?.slave_ratio,
      )
    )
      actions.push(() => setStepper4Slave(d.addon_motor_4_state!.slave_ratio!));

    // Addon Motor 5
    if (
      d?.addon_motor_5_state?.enabled !== undefined &&
      d.addon_motor_5_state.enabled !== currentData.addon_motor_5_state?.enabled
    )
      actions.push(() =>
        setStepper5Mode(d.addon_motor_5_state!.enabled ? "Run" : "Standby"),
      );
    if (
      changed(
        d?.addon_motor_5_state?.forward,
        currentData.addon_motor_5_state?.forward,
      )
    )
      actions.push(() => setStepper5Forward(d.addon_motor_5_state!.forward!));
    if (
      changed(
        d?.addon_motor_5_state?.master_ratio,
        currentData.addon_motor_5_state?.master_ratio,
      )
    )
      actions.push(() =>
        setStepper5Master(d.addon_motor_5_state!.master_ratio!),
      );
    if (
      changed(
        d?.addon_motor_5_state?.slave_ratio,
        currentData.addon_motor_5_state?.slave_ratio,
      )
    )
      actions.push(() => setStepper5Slave(d.addon_motor_5_state!.slave_ratio!));

    // Slave Puller
    if (
      changed(
        d?.slave_puller_state?.enabled,
        currentData.slave_puller_state?.enabled,
      )
    )
      actions.push(() => setSlavePullerEnabled(d.slave_puller_state!.enabled!));
    if (
      changed(
        d?.slave_puller_state?.forward,
        currentData.slave_puller_state?.forward,
      )
    )
      actions.push(() => setSlavePullerForward(d.slave_puller_state!.forward!));
    if (
      changed(
        d?.slave_puller_state?.target_angle,
        currentData.slave_puller_state?.target_angle,
      )
    )
      actions.push(() =>
        setSlavePullerTargetAngle(d.slave_puller_state!.target_angle!),
      );
    if (
      changed(
        d?.slave_puller_state?.sensitivity,
        currentData.slave_puller_state?.sensitivity,
      )
    )
      actions.push(() =>
        setSlavePullerSensitivity(d.slave_puller_state!.sensitivity!),
      );
    if (
      changed(
        d?.slave_puller_state?.min_speed_factor,
        currentData.slave_puller_state?.min_speed_factor,
      )
    )
      actions.push(() =>
        setSlavePullerMinSpeedFactor(d.slave_puller_state!.min_speed_factor!),
      );
    if (
      changed(
        d?.slave_puller_state?.max_speed_factor,
        currentData.slave_puller_state?.max_speed_factor,
      )
    )
      actions.push(() =>
        setSlavePullerMaxSpeedFactor(d.slave_puller_state!.max_speed_factor!),
      );

    // Heating PID (each zone sends all 3 PID values together)
    for (const zone of [
      "zone_1",
      "zone_2",
      "zone_3",
      "zone_4",
      "zone_5",
      "zone_6",
    ] as const) {
      const presetZone = d?.heating_pid_settings?.[zone];
      const currentZone = currentData.heating_pid_settings?.[zone];
      if (
        presetZone &&
        (changed(presetZone.kp, currentZone?.kp) ||
          changed(presetZone.ki, currentZone?.ki) ||
          changed(presetZone.kd, currentZone?.kd))
      ) {
        const kp = presetZone.kp ?? currentZone?.kp ?? 0;
        const ki = presetZone.ki ?? currentZone?.ki ?? 0;
        const kd = presetZone.kd ?? currentZone?.kd ?? 0;
        actions.push(() => setHeatingPid(zone, kp, ki, kd));
      }
    }

    // Winder Tension Arm Monitor
    if (
      changed(
        d?.winder_tension_arm_monitor_state?.enabled,
        currentData.winder_tension_arm_monitor_state?.enabled,
      )
    )
      actions.push(() =>
        setWinderTensionArmMonitorEnabled(
          d.winder_tension_arm_monitor_state!.enabled!,
        ),
      );
    if (
      changed(
        d?.winder_tension_arm_monitor_state?.min_angle,
        currentData.winder_tension_arm_monitor_state?.min_angle,
      )
    )
      actions.push(() =>
        setWinderTensionArmMonitorMinAngle(
          d.winder_tension_arm_monitor_state!.min_angle!,
        ),
      );
    if (
      changed(
        d?.winder_tension_arm_monitor_state?.max_angle,
        currentData.winder_tension_arm_monitor_state?.max_angle,
      )
    )
      actions.push(() =>
        setWinderTensionArmMonitorMaxAngle(
          d.winder_tension_arm_monitor_state!.max_angle!,
        ),
      );

    // Addon Tension Arm Monitor
    if (
      changed(
        d?.addon_tension_arm_monitor_state?.enabled,
        currentData.addon_tension_arm_monitor_state?.enabled,
      )
    )
      actions.push(() =>
        setAddonTensionArmMonitorEnabled(
          d.addon_tension_arm_monitor_state!.enabled!,
        ),
      );
    if (
      changed(
        d?.addon_tension_arm_monitor_state?.min_angle,
        currentData.addon_tension_arm_monitor_state?.min_angle,
      )
    )
      actions.push(() =>
        setAddonTensionArmMonitorMinAngle(
          d.addon_tension_arm_monitor_state!.min_angle!,
        ),
      );
    if (
      changed(
        d?.addon_tension_arm_monitor_state?.max_angle,
        currentData.addon_tension_arm_monitor_state?.max_angle,
      )
    )
      actions.push(() =>
        setAddonTensionArmMonitorMaxAngle(
          d.addon_tension_arm_monitor_state!.max_angle!,
        ),
      );

    // Slave Tension Arm Monitor
    if (
      changed(
        d?.slave_tension_arm_monitor_state?.enabled,
        currentData.slave_tension_arm_monitor_state?.enabled,
      )
    )
      actions.push(() =>
        setSlaveTensionArmMonitorEnabled(
          d.slave_tension_arm_monitor_state!.enabled!,
        ),
      );
    if (
      changed(
        d?.slave_tension_arm_monitor_state?.min_angle,
        currentData.slave_tension_arm_monitor_state?.min_angle,
      )
    )
      actions.push(() =>
        setSlaveTensionArmMonitorMinAngle(
          d.slave_tension_arm_monitor_state!.min_angle!,
        ),
      );
    if (
      changed(
        d?.slave_tension_arm_monitor_state?.max_angle,
        currentData.slave_tension_arm_monitor_state?.max_angle,
      )
    )
      actions.push(() =>
        setSlaveTensionArmMonitorMaxAngle(
          d.slave_tension_arm_monitor_state!.max_angle!,
        ),
      );

    // Optris 1 Monitor
    if (
      changed(
        d?.optris_1_monitor_state?.enabled,
        currentData.optris_1_monitor_state?.enabled,
      )
    )
      actions.push(() =>
        setOptris1MonitorEnabled(d.optris_1_monitor_state!.enabled!),
      );
    if (
      changed(
        d?.optris_1_monitor_state?.min_voltage,
        currentData.optris_1_monitor_state?.min_voltage,
      )
    )
      actions.push(() =>
        setOptris1MonitorMinVoltage(d.optris_1_monitor_state!.min_voltage!),
      );
    if (
      changed(
        d?.optris_1_monitor_state?.max_voltage,
        currentData.optris_1_monitor_state?.max_voltage,
      )
    )
      actions.push(() =>
        setOptris1MonitorMaxVoltage(d.optris_1_monitor_state!.max_voltage!),
      );

    // Optris 2 Monitor
    if (
      changed(
        d?.optris_2_monitor_state?.enabled,
        currentData.optris_2_monitor_state?.enabled,
      )
    )
      actions.push(() =>
        setOptris2MonitorEnabled(d.optris_2_monitor_state!.enabled!),
      );
    if (
      changed(
        d?.optris_2_monitor_state?.min_voltage,
        currentData.optris_2_monitor_state?.min_voltage,
      )
    )
      actions.push(() =>
        setOptris2MonitorMinVoltage(d.optris_2_monitor_state!.min_voltage!),
      );
    if (
      changed(
        d?.optris_2_monitor_state?.max_voltage,
        currentData.optris_2_monitor_state?.max_voltage,
      )
    )
      actions.push(() =>
        setOptris2MonitorMaxVoltage(d.optris_2_monitor_state!.max_voltage!),
      );

    // Sleep Timer
    if (
      changed(
        d?.sleep_timer_state?.enabled,
        currentData.sleep_timer_state?.enabled,
      )
    )
      actions.push(() => setSleepTimerEnabled(d.sleep_timer_state!.enabled!));
    if (
      changed(
        d?.sleep_timer_state?.timeout_seconds,
        currentData.sleep_timer_state?.timeout_seconds,
      )
    )
      actions.push(() =>
        setSleepTimerTimeout(d.sleep_timer_state!.timeout_seconds!),
      );

    // Execute actions sequentially, spread evenly over 5 seconds
    if (actions.length === 0) {
      toast.success("Preset already active", {
        description: "All values already match the preset.",
      });
      return;
    }

    const toastId = toast.loading(`Applying preset... (0/${actions.length})`, {
      description: "Please wait while settings are applied.",
    });

    const delay = Math.min(5000 / actions.length, 200);
    for (let i = 0; i < actions.length; i++) {
      actions[i]();
      toast.loading(`Applying preset... (${i + 1}/${actions.length})`, {
        id: toastId,
        description: "Please wait while settings are applied.",
      });
      if (i < actions.length - 1) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }

    toast.success("Preset applied", {
      id: toastId,
      description: `${actions.length} setting${actions.length > 1 ? "s" : ""} applied successfully.`,
    });
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
    heating_pid_settings: s?.heating_pid_settings
      ? {
          zone_1: {
            kp: s.heating_pid_settings.zone_1?.kp,
            ki: s.heating_pid_settings.zone_1?.ki,
            kd: s.heating_pid_settings.zone_1?.kd,
          },
          zone_2: {
            kp: s.heating_pid_settings.zone_2?.kp,
            ki: s.heating_pid_settings.zone_2?.ki,
            kd: s.heating_pid_settings.zone_2?.kd,
          },
          zone_3: {
            kp: s.heating_pid_settings.zone_3?.kp,
            ki: s.heating_pid_settings.zone_3?.ki,
            kd: s.heating_pid_settings.zone_3?.kd,
          },
          zone_4: {
            kp: s.heating_pid_settings.zone_4?.kp,
            ki: s.heating_pid_settings.zone_4?.ki,
            kd: s.heating_pid_settings.zone_4?.kd,
          },
          zone_5: {
            kp: s.heating_pid_settings.zone_5?.kp,
            ki: s.heating_pid_settings.zone_5?.ki,
            kd: s.heating_pid_settings.zone_5?.kd,
          },
          zone_6: {
            kp: s.heating_pid_settings.zone_6?.kp,
            ki: s.heating_pid_settings.zone_6?.ki,
            kd: s.heating_pid_settings.zone_6?.kd,
          },
        }
      : {},
    winder_tension_arm_monitor_state: s?.winder_tension_arm_monitor_state
      ? {
          enabled: s.winder_tension_arm_monitor_state.enabled,
          min_angle: s.winder_tension_arm_monitor_state.min_angle,
          max_angle: s.winder_tension_arm_monitor_state.max_angle,
        }
      : {},
    addon_tension_arm_monitor_state: s?.addon_tension_arm_monitor_state
      ? {
          enabled: s.addon_tension_arm_monitor_state.enabled,
          min_angle: s.addon_tension_arm_monitor_state.min_angle,
          max_angle: s.addon_tension_arm_monitor_state.max_angle,
        }
      : {},
    slave_tension_arm_monitor_state: s?.slave_tension_arm_monitor_state
      ? {
          enabled: s.slave_tension_arm_monitor_state.enabled,
          min_angle: s.slave_tension_arm_monitor_state.min_angle,
          max_angle: s.slave_tension_arm_monitor_state.max_angle,
        }
      : {},
    optris_1_monitor_state: s?.optris_1_monitor_state
      ? {
          enabled: s.optris_1_monitor_state.enabled,
          min_voltage: s.optris_1_monitor_state.min_voltage,
          max_voltage: s.optris_1_monitor_state.max_voltage,
        }
      : {},
    optris_2_monitor_state: s?.optris_2_monitor_state
      ? {
          enabled: s.optris_2_monitor_state.enabled,
          min_voltage: s.optris_2_monitor_state.min_voltage,
          max_voltage: s.optris_2_monitor_state.max_voltage,
        }
      : {},
    sleep_timer_state: s?.sleep_timer_state
      ? {
          enabled: s.sleep_timer_state.enabled,
          timeout_seconds: s.sleep_timer_state.timeout_seconds,
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
