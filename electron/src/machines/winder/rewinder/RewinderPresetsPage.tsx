import { PresetsPage } from "@/components/preset/PresetsPage";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";
import { Preset } from "@/lib/preset/preset";
import { rewinder } from "@/machines/properties";
import { z } from "zod";
import {
  prepareControlStateSchema,
  pullerStateSchema,
  rewindAutomaticActionStateSchema,
  sourceSpoolStateSchema,
  takeupSpoolStateSchema,
  tensionArmControlStateSchema,
  traverseStateSchema,
} from "./rewinderNamespace";
import { useRewinder } from "./useRewinder";

const rewinderPresetDataSchema = z
  .object({
    traverse_state: traverseStateSchema
      .pick({
        limit_inner: true,
        limit_outer: true,
        start_position: true,
        step_size: true,
        padding: true,
        laserpointer: true,
      })
      .partial(),
    puller_state: pullerStateSchema.partial(),
    takeup_spool_state: takeupSpoolStateSchema.partial(),
    source_spool_state: sourceSpoolStateSchema.partial(),
    takeup_tension_arm_control_state: tensionArmControlStateSchema.partial(),
    source_tension_arm_control_state: tensionArmControlStateSchema.partial(),
    prepare_control_state: prepareControlStateSchema.partial(),
    rewind_automatic_action_state: rewindAutomaticActionStateSchema.partial(),
  })
  .partial();

type RewinderPresetData = z.infer<typeof rewinderPresetDataSchema>;

const schemas = new Map([[1, rewinderPresetDataSchema]]);

const renderNumber = (value: number | undefined, digits = 1) =>
  value?.toFixed(digits);

const previewEntries: PresetPreviewEntries<RewinderPresetData> = [
  {
    name: "Line Speed",
    unit: "m/min",
    renderValue: (data) => renderNumber(data.puller_state?.target_speed, 2),
  },
  {
    name: "Required Length",
    unit: "m",
    renderValue: (data) =>
      renderNumber(data.rewind_automatic_action_state?.required_meters, 1),
  },
  {
    name: "After Length",
    renderValue: (data) => data.rewind_automatic_action_state?.mode,
  },
  previewSeparator,
  {
    name: "Takeup Diameter",
    unit: "mm",
    renderValue: (data) =>
      renderNumber(data.takeup_spool_state?.diameter_mm ?? undefined, 0),
  },
  {
    name: "Source Diameter",
    unit: "mm",
    renderValue: (data) =>
      renderNumber(data.source_spool_state?.diameter_mm ?? undefined, 0),
  },
  {
    name: "Takeup Algorithm",
    renderValue: (data) => data.takeup_spool_state?.regulation_mode,
  },
  {
    name: "Takeup Tension Target",
    renderValue: (data) =>
      renderNumber(data.takeup_spool_state?.adaptive_tension_target, 2),
  },
  {
    name: "Source Tension Target",
    renderValue: (data) =>
      renderNumber(data.source_spool_state?.adaptive_tension_target, 2),
  },
  previewSeparator,
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (data) => renderNumber(data.traverse_state?.limit_inner, 1),
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (data) => renderNumber(data.traverse_state?.limit_outer, 1),
  },
  {
    name: "Traverse Start",
    unit: "mm",
    renderValue: (data) => renderNumber(data.traverse_state?.start_position, 1),
  },
  {
    name: "Traverse Step",
    unit: "mm",
    renderValue: (data) => renderNumber(data.traverse_state?.step_size, 2),
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (data) => renderNumber(data.traverse_state?.padding, 2),
  },
];

export function RewinderPresetsPage() {
  const {
    state,
    defaultState,
    isLoading,
    settingsEditPermitted,
    setPullerTargetSpeed,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    setTakeupSpoolDiameter,
    setSourceSpoolDiameter,
    setTakeupSpoolRegulationMode,
    setTakeupSpoolMinMaxMinSpeed,
    setTakeupSpoolMinMaxMaxSpeed,
    setTakeupTensionTarget,
    setTakeupSpoolAdaptiveRadiusLearningRate,
    setTakeupSpoolAdaptiveMaxSpeedMultiplier,
    setTakeupSpoolAdaptiveAccelerationFactor,
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setSourceTensionTarget,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStartPosition,
    setTraverseStepSize,
    setTraversePadding,
    enableTraverseLaserpointer,
    setTakeupTensionArmControl,
    setSourceTensionArmControl,
    setPrepareControl,
  } = useRewinder();

  const applyPreset = (preset: Preset<RewinderPresetData>) => {
    if (!settingsEditPermitted || isLoading) {
      return;
    }

    const data = preset.data ?? {};
    const traverse = data.traverse_state ?? {};
    const puller = data.puller_state ?? {};
    const automatic = data.rewind_automatic_action_state ?? {};
    const takeup = data.takeup_spool_state ?? {};
    const source = data.source_spool_state ?? {};
    const takeupArm = data.takeup_tension_arm_control_state ?? {};
    const sourceArm = data.source_tension_arm_control_state ?? {};
    const prepare = data.prepare_control_state ?? {};
    const defaults = defaultState;

    setPullerTargetSpeed(
      puller.target_speed ?? defaults?.puller_state.target_speed ?? 10.0,
    );
    setRewindAutomaticRequiredMeters(
      automatic.required_meters ??
        defaults?.rewind_automatic_action_state.required_meters ??
        100.0,
    );
    setRewindAutomaticAction(
      automatic.mode ??
        defaults?.rewind_automatic_action_state.mode ??
        "NoAction",
    );

    setTakeupSpoolDiameter(
      takeup.diameter_mm ?? defaults?.takeup_spool_state.diameter_mm ?? 100,
    );
    setSourceSpoolDiameter(
      source.diameter_mm ?? defaults?.source_spool_state.diameter_mm ?? 100,
    );
    setTakeupSpoolRegulationMode(
      takeup.regulation_mode ??
        defaults?.takeup_spool_state.regulation_mode ??
        "Adaptive",
    );
    setTakeupSpoolMinMaxMinSpeed(
      takeup.minmax_min_speed ??
        defaults?.takeup_spool_state.minmax_min_speed ??
        5.0,
    );
    setTakeupSpoolMinMaxMaxSpeed(
      takeup.minmax_max_speed ??
        defaults?.takeup_spool_state.minmax_max_speed ??
        50.0,
    );
    setTakeupTensionTarget(
      takeup.adaptive_tension_target ??
        defaults?.takeup_spool_state.adaptive_tension_target ??
        0.5,
    );
    setTakeupSpoolAdaptiveRadiusLearningRate(
      takeup.adaptive_radius_learning_rate ??
        defaults?.takeup_spool_state.adaptive_radius_learning_rate ??
        0.1,
    );
    setTakeupSpoolAdaptiveMaxSpeedMultiplier(
      takeup.adaptive_max_speed_multiplier ??
        defaults?.takeup_spool_state.adaptive_max_speed_multiplier ??
        2.0,
    );
    setTakeupSpoolAdaptiveAccelerationFactor(
      takeup.adaptive_acceleration_factor ??
        defaults?.takeup_spool_state.adaptive_acceleration_factor ??
        1.5,
    );
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier(
      takeup.adaptive_deacceleration_urgency_multiplier ??
        defaults?.takeup_spool_state
          .adaptive_deacceleration_urgency_multiplier ??
        2.0,
    );
    setSourceTensionTarget(
      source.adaptive_tension_target ??
        defaults?.source_spool_state.adaptive_tension_target ??
        0.5,
    );

    setTraverseLimitInner(
      traverse.limit_inner ?? defaults?.traverse_state.limit_inner ?? 22.0,
    );
    setTraverseLimitOuter(
      traverse.limit_outer ?? defaults?.traverse_state.limit_outer ?? 92.0,
    );
    setTraverseStartPosition(
      traverse.start_position ??
        defaults?.traverse_state.start_position ??
        92.0,
    );
    setTraverseStepSize(
      traverse.step_size ?? defaults?.traverse_state.step_size ?? 5.0,
    );
    setTraversePadding(
      traverse.padding ?? defaults?.traverse_state.padding ?? 10.0,
    );
    enableTraverseLaserpointer(
      traverse.laserpointer ?? defaults?.traverse_state.laserpointer ?? false,
    );

    setTakeupTensionArmControl(
      "hard_min_angle",
      takeupArm.hard_min_angle ??
        defaults?.takeup_tension_arm_control_state.hard_min_angle ??
        15,
    );
    setTakeupTensionArmControl(
      "hard_max_angle",
      takeupArm.hard_max_angle ??
        defaults?.takeup_tension_arm_control_state.hard_max_angle ??
        85,
    );
    setTakeupTensionArmControl(
      "start_min_angle",
      takeupArm.start_min_angle ??
        defaults?.takeup_tension_arm_control_state.start_min_angle ??
        35,
    );
    setTakeupTensionArmControl(
      "start_max_angle",
      takeupArm.start_max_angle ??
        defaults?.takeup_tension_arm_control_state.start_max_angle ??
        65,
    );
    setTakeupTensionArmControl(
      "target_angle",
      takeupArm.target_angle ??
        defaults?.takeup_tension_arm_control_state.target_angle ??
        50,
    );
    setSourceTensionArmControl(
      "hard_min_angle",
      sourceArm.hard_min_angle ??
        defaults?.source_tension_arm_control_state.hard_min_angle ??
        20,
    );
    setSourceTensionArmControl(
      "hard_max_angle",
      sourceArm.hard_max_angle ??
        defaults?.source_tension_arm_control_state.hard_max_angle ??
        90,
    );
    setSourceTensionArmControl(
      "start_min_angle",
      sourceArm.start_min_angle ??
        defaults?.source_tension_arm_control_state.start_min_angle ??
        35,
    );
    setSourceTensionArmControl(
      "start_max_angle",
      sourceArm.start_max_angle ??
        defaults?.source_tension_arm_control_state.start_max_angle ??
        70,
    );
    setSourceTensionArmControl(
      "target_angle",
      sourceArm.target_angle ??
        defaults?.source_tension_arm_control_state.target_angle ??
        55,
    );
    setPrepareControl(
      "tolerance_angle",
      prepare.tolerance_angle ??
        defaults?.prepare_control_state.tolerance_angle ??
        3.0,
    );
    setPrepareControl(
      "settle_rate",
      prepare.settle_rate ?? defaults?.prepare_control_state.settle_rate ?? 0.5,
    );
  };

  const toPresetData = (s: typeof state): RewinderPresetData => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      start_position: s?.traverse_state?.start_position,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state ?? {},
    takeup_spool_state: s?.takeup_spool_state ?? {},
    source_spool_state: s?.source_spool_state ?? {},
    takeup_tension_arm_control_state: s?.takeup_tension_arm_control_state ?? {},
    source_tension_arm_control_state: s?.source_tension_arm_control_state ?? {},
    prepare_control_state: s?.prepare_control_state ?? {},
    rewind_automatic_action_state: s?.rewind_automatic_action_state ?? {},
  });

  return (
    <PresetsPage
      machine_identification={rewinder.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      defaultState={toPresetData(defaultState)}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      applyDisabled={!settingsEditPermitted || isLoading}
    />
  );
}
