import { PresetsPage } from "@/components/preset/PresetsPage";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";
import { Preset } from "@/lib/preset/preset";
import { rewinder } from "@/machines/properties";
import React from "react";
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
        start: true,
        start_position: true,
        custom_start_position: true,
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

const presetValue = <T,>(
  presetValue: T | null | undefined,
  defaultValue: T | null | undefined,
  fallback: T,
) => presetValue ?? defaultValue ?? fallback;

const presetFallback = {
  targetSpeed: 10.0,
  requiredMeters: 100.0,
  takeupDiameter: 100,
  sourceDiameter: 100,
  traverseInner: 22.0,
  traverseOuter: 92.0,
  traverseStart: 92.0,
  traverseStep: 5.0,
  traversePadding: 10.0,
  takeupArm: {
    hardMin: 15,
    hardMax: 85,
    startMin: 35,
    startMax: 65,
    target: 50,
  },
  sourceArm: {
    hardMin: 20,
    hardMax: 90,
    startMin: 35,
    startMax: 70,
    target: 55,
  },
  prepareTolerance: 3.0,
  prepareSettleRate: 0.5,
} as const;

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
    name: "Traverse Start Side",
    renderValue: (data) => data.traverse_state?.start,
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
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStart,
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

    const applyTakeupArm = (
      field: keyof NonNullable<
        RewinderPresetData["takeup_tension_arm_control_state"]
      >,
      fallback: number,
    ) =>
      setTakeupTensionArmControl(
        field,
        presetValue(
          takeupArm[field],
          defaults?.takeup_tension_arm_control_state[field],
          fallback,
        ),
      );
    const applySourceArm = (
      field: keyof NonNullable<
        RewinderPresetData["source_tension_arm_control_state"]
      >,
      fallback: number,
    ) =>
      setSourceTensionArmControl(
        field,
        presetValue(
          sourceArm[field],
          defaults?.source_tension_arm_control_state[field],
          fallback,
        ),
      );
    const applyPrepare = (
      field: keyof NonNullable<RewinderPresetData["prepare_control_state"]>,
      fallback: number,
    ) =>
      setPrepareControl(
        field,
        presetValue(
          prepare[field],
          defaults?.prepare_control_state[field],
          fallback,
        ),
      );

    setPullerTargetSpeed(
      presetValue(
        puller.target_speed,
        defaults?.puller_state.target_speed,
        presetFallback.targetSpeed,
      ),
    );
    setRewindAutomaticRequiredMeters(
      presetValue(
        automatic.required_meters,
        defaults?.rewind_automatic_action_state.required_meters,
        presetFallback.requiredMeters,
      ),
    );
    setRewindAutomaticAction(
      presetValue(
        automatic.mode,
        defaults?.rewind_automatic_action_state.mode,
        "NoAction",
      ),
    );

    setTakeupSpoolDiameter(
      presetValue(
        takeup.diameter_mm,
        defaults?.takeup_spool_state.diameter_mm,
        presetFallback.takeupDiameter,
      ),
    );
    setSourceSpoolDiameter(
      presetValue(
        source.diameter_mm,
        defaults?.source_spool_state.diameter_mm,
        presetFallback.sourceDiameter,
      ),
    );
    setTraverseLimitInner(
      presetValue(
        traverse.limit_inner,
        defaults?.traverse_state.limit_inner,
        presetFallback.traverseInner,
      ),
    );
    setTraverseLimitOuter(
      presetValue(
        traverse.limit_outer,
        defaults?.traverse_state.limit_outer,
        presetFallback.traverseOuter,
      ),
    );
    const start = presetValue(
      traverse.start,
      defaults?.traverse_state.start,
      "Left",
    );
    if (start === "Custom" || (!traverse.start && traverse.start_position)) {
      setTraverseStartPosition(
        presetValue(
          traverse.custom_start_position ?? traverse.start_position,
          defaults?.traverse_state.custom_start_position,
          presetFallback.traverseStart,
        ),
      );
    } else {
      setTraverseStart(start);
    }
    setTraverseStepSize(
      presetValue(
        traverse.step_size,
        defaults?.traverse_state.step_size,
        presetFallback.traverseStep,
      ),
    );
    setTraversePadding(
      presetValue(
        traverse.padding,
        defaults?.traverse_state.padding,
        presetFallback.traversePadding,
      ),
    );
    enableTraverseLaserpointer(
      presetValue(
        traverse.laserpointer,
        defaults?.traverse_state.laserpointer,
        false,
      ),
    );

    applyTakeupArm("hard_min_angle", presetFallback.takeupArm.hardMin);
    applyTakeupArm("hard_max_angle", presetFallback.takeupArm.hardMax);
    applyTakeupArm("start_min_angle", presetFallback.takeupArm.startMin);
    applyTakeupArm("start_max_angle", presetFallback.takeupArm.startMax);
    applyTakeupArm("target_angle", presetFallback.takeupArm.target);
    applySourceArm("hard_min_angle", presetFallback.sourceArm.hardMin);
    applySourceArm("hard_max_angle", presetFallback.sourceArm.hardMax);
    applySourceArm("start_min_angle", presetFallback.sourceArm.startMin);
    applySourceArm("start_max_angle", presetFallback.sourceArm.startMax);
    applySourceArm("target_angle", presetFallback.sourceArm.target);
    applyPrepare("tolerance_angle", presetFallback.prepareTolerance);
    applyPrepare("settle_rate", presetFallback.prepareSettleRate);
  };

  const toPresetData = (s: typeof state): RewinderPresetData => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      start: s?.traverse_state?.start,
      start_position: s?.traverse_state?.start_position,
      custom_start_position: s?.traverse_state?.custom_start_position,
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
    />
  );
}
