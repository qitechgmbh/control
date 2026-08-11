import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React, { useState } from "react";
import {
  ExtruderMixerPreset,
  saveExtruderMixerPresets,
  setActiveExtruderMixerPreset,
  useExtruderMixerStorage,
} from "./extruderMixerConfig";

const newPresetDefaults: Omit<ExtruderMixerPreset, "id"> = {
  name: "New production preset",
  dosageA: 1,
  dosageB: 1,
  screwRpm: 20,
  frontTemperature: 180,
  middleTemperature: 185,
  backTemperature: 180,
  nozzleTemperature: 185,
};

function NumberField({
  label,
  value,
  unit,
  min,
  max,
  step = 1,
  onChange,
}: {
  label: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
}) {
  return (
    <Label label={label}>
      <div className="relative">
        <input
          type="number"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(event) => onChange(Number(event.target.value))}
          className="h-12 w-full rounded-md border border-gray-200 px-3 pr-14"
        />
        <span className="absolute top-3 right-3 text-sm text-gray-500">
          {unit}
        </span>
      </div>
    </Label>
  );
}

export function ExtruderMixerPresetsPage() {
  const { presets, activePreset } = useExtruderMixerStorage();
  const [draft, setDraft] = useState(newPresetDefaults);

  const setNumber = (
    key: keyof Omit<ExtruderMixerPreset, "id" | "name">,
    value: number,
  ) => setDraft((current) => ({ ...current, [key]: value }));

  const save = () => {
    const dosageA = Math.min(100, Math.max(0, draft.dosageA));
    const dosageB = Math.min(100, Math.max(0, draft.dosageB));
    const preset: ExtruderMixerPreset = {
      ...draft,
      dosageA,
      dosageB,
      id: `${Date.now()}-${draft.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`,
    };
    saveExtruderMixerPresets([...presets, preset]);
    setActiveExtruderMixerPreset(preset.id);
  };

  const remove = (id: string) => {
    if (presets.length <= 1) return;
    const next = presets.filter((preset) => preset.id !== id);
    saveExtruderMixerPresets(next);
    if (activePreset.id === id) setActiveExtruderMixerPreset(next[0].id);
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Create Combined Preset">
          <Label label="Preset Name">
            <input
              value={draft.name}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  name: event.target.value,
                }))
              }
              className="h-12 w-full rounded-md border border-gray-200 px-3"
            />
          </Label>
          <div className="grid grid-cols-2 gap-3">
            <NumberField
              label="Extruder Screw"
              value={draft.screwRpm}
              unit="rpm"
              min={0}
              max={100}
              onChange={(value) => setNumber("screwRpm", value)}
            />
            <NumberField
              label="Hopper A / Main"
              value={draft.dosageA}
              unit="%"
              min={0}
              max={100}
              step={0.1}
              onChange={(value) =>
                setNumber("dosageA", Math.min(100, Math.max(0, value)))
              }
            />
            <NumberField
              label="Hopper B / Main"
              value={draft.dosageB}
              unit="%"
              min={0}
              max={100}
              step={0.1}
              onChange={(value) =>
                setNumber("dosageB", Math.min(100, Math.max(0, value)))
              }
            />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <NumberField
              label="Front Temperature"
              value={draft.frontTemperature}
              unit="°C"
              min={0}
              max={300}
              onChange={(value) => setNumber("frontTemperature", value)}
            />
            <NumberField
              label="Middle Temperature"
              value={draft.middleTemperature}
              unit="°C"
              min={0}
              max={300}
              onChange={(value) => setNumber("middleTemperature", value)}
            />
            <NumberField
              label="Back Temperature"
              value={draft.backTemperature}
              unit="°C"
              min={0}
              max={300}
              onChange={(value) => setNumber("backTemperature", value)}
            />
            <NumberField
              label="Nozzle Temperature"
              value={draft.nozzleTemperature}
              unit="°C"
              min={0}
              max={300}
              onChange={(value) => setNumber("nozzleTemperature", value)}
            />
          </div>
          <TouchButton
            icon="lu:Save"
            disabled={!draft.name.trim()}
            onClick={save}
          >
            Save Preset
          </TouchButton>
        </ControlCard>

        <ControlCard title="Saved Presets">
          {presets.map((preset) => {
            const active = preset.id === activePreset.id;
            return (
              <div
                key={preset.id}
                className={`rounded-xl border p-4 ${
                  active
                    ? "border-green-300 bg-green-50"
                    : "border-gray-200 bg-gray-50"
                }`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <strong>{preset.name}</strong>
                    <p className="mt-1 text-sm text-gray-500">
                      Main 100% reference · A {preset.dosageA}% of main · B{" "}
                      {preset.dosageB}% of main
                    </p>
                    <p className="text-xs text-gray-500">
                      Screw {preset.screwRpm} rpm · {preset.frontTemperature}/
                      {preset.middleTemperature}/{preset.backTemperature}/
                      {preset.nozzleTemperature} °C
                    </p>
                  </div>
                  {active && (
                    <span className="rounded-full bg-green-600 px-2 py-1 text-xs font-semibold text-white">
                      ACTIVE
                    </span>
                  )}
                </div>
                <div className="mt-3 flex gap-2">
                  <TouchButton
                    variant="outline"
                    disabled={active}
                    onClick={() => setActiveExtruderMixerPreset(preset.id)}
                  >
                    Apply
                  </TouchButton>
                  <TouchButton
                    variant="outline"
                    disabled={presets.length <= 1}
                    onClick={() => remove(preset.id)}
                  >
                    Delete
                  </TouchButton>
                </div>
              </div>
            );
          })}
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
