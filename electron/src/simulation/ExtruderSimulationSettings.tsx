import React, { useState } from "react";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruderSimulation } from "./useExtruderSimulation";
import { ZoneName } from "@/client/simulationNamespace";

const ZONES: { key: ZoneName; label: string }[] = [
  { key: "front", label: "Front" },
  { key: "middle", label: "Middle" },
  { key: "back", label: "Back" },
  { key: "nozzle", label: "Nozzle" },
];

export function ExtruderSimulationSettings() {
  const {
    state,
    setSetpoint,
    setAlgorithm,
    setPidGain,
    setObserverPiGain,
    setScrewRpm,
    setSpeed,
    play,
    pause,
    reset,
  } = useExtruderSimulation();

  const [showGains, setShowGains] = useState(false);
  const [resetTemperature, setResetTemperature] = useState(22);

  const algorithm: "pid" | "observer-pi" | undefined = state?.strategy.Pid
    ? "pid"
    : state?.strategy.ObserverPi
      ? "observer-pi"
      : undefined;

  return (
    <div className="flex flex-col gap-6">
      <ControlCard title="Zone Setpoints">
        <ControlGrid columns={2}>
          {ZONES.map(({ key, label }) => (
            <Label key={key} label={label}>
              <EditValue
                value={state?.setpoints_c[key]}
                defaultValue={180}
                unit="C"
                title={`${label} Setpoint`}
                min={0}
                max={300}
                renderValue={(v) => roundToDecimals(v, 0)}
                onChange={(v) => setSetpoint(key, v)}
              />
            </Label>
          ))}
        </ControlGrid>
      </ControlCard>

      <ControlCard title="Algorithm">
        <Label label="Control Law">
          <SelectionGroup<"pid" | "observer-pi">
            value={algorithm}
            options={{
              pid: { children: "PID" },
              "observer-pi": { children: "Observer PI" },
            }}
            onChange={setAlgorithm}
          />
        </Label>
        <Label label="Show Gains">
          <SelectionGroupBoolean
            value={showGains}
            optionTrue={{ children: "Show" }}
            optionFalse={{ children: "Hide" }}
            onChange={setShowGains}
          />
        </Label>
      </ControlCard>

      {showGains && algorithm === "pid" && (
        <ControlGrid columns={2}>
          {ZONES.map(({ key, label }) => (
            <ControlCard key={key} title={`${label} PID Gains`}>
              {(["kp", "ki", "kd"] as const).map((gain) => (
                <Label key={gain} label={gain.toUpperCase()}>
                  <EditValue
                    value={state?.strategy.Pid?.[zoneIndex(key)][gain]}
                    min={0}
                    max={10}
                    step={0.0001}
                    renderValue={(v) => roundToDecimals(v, 5)}
                    onChange={(v) => setPidGain(key, gain, v)}
                    title={`${label} ${gain.toUpperCase()}`}
                  />
                </Label>
              ))}
            </ControlCard>
          ))}
        </ControlGrid>
      )}

      {showGains && algorithm === "observer-pi" && (
        <ControlGrid columns={2}>
          {ZONES.map(({ key, label }) => (
            <ControlCard key={key} title={`${label} Observer PI Gains`}>
              {(["kp", "ki"] as const).map((gain) => (
                <Label key={gain} label={gain.toUpperCase()}>
                  <EditValue
                    value={state?.strategy.ObserverPi?.[zoneIndex(key)][gain]}
                    min={0}
                    max={10}
                    step={0.0001}
                    renderValue={(v) => roundToDecimals(v, 5)}
                    onChange={(v) => setObserverPiGain(key, gain, v)}
                    title={`${label} ${gain.toUpperCase()}`}
                  />
                </Label>
              ))}
            </ControlCard>
          ))}
        </ControlGrid>
      )}

      <ControlCard title="Extrusion">
        <Label label="Screw Speed">
          <EditValue
            value={state?.screw_rpm}
            defaultValue={0}
            unit="rpm"
            title="Screw Speed"
            description="Above 0 rpm, the polymer melt is modelled and pulls heat out of the barrel."
            min={0}
            max={100}
            renderValue={(v) => roundToDecimals(v, 0)}
            onChange={setScrewRpm}
          />
        </Label>
      </ControlCard>

      <ControlCard title="Playback">
        <Label label="Run State">
          <SelectionGroupBoolean
            value={state?.running}
            optionTrue={{ children: "Running" }}
            optionFalse={{ children: "Paused" }}
            onChange={(running) => (running ? play() : pause())}
          />
        </Label>
        <Label label="Speed">
          <EditValue
            value={state?.speed}
            defaultValue={1}
            title="Simulation Speed"
            description="Simulated seconds per wall-clock second."
            min={0.1}
            max={200}
            step={1}
            renderValue={(v) => `${roundToDecimals(v, 1)}x`}
            onChange={setSpeed}
          />
        </Label>
        <Label label="Reset">
          <div className="flex flex-row items-end gap-4">
            <EditValue
              value={resetTemperature}
              defaultValue={22}
              unit="C"
              title="Reset Temperature"
              min={0}
              max={300}
              renderValue={(v) => roundToDecimals(v, 0)}
              onChange={setResetTemperature}
            />
            <button
              onClick={() => reset(resetTemperature)}
              className="inline-block h-min w-fit max-w-max rounded bg-red-600 px-4 py-4 text-base whitespace-nowrap text-white hover:bg-red-700"
            >
              Reset Simulation
            </button>
          </div>
        </Label>
      </ControlCard>
    </div>
  );
}

function zoneIndex(zone: ZoneName): 0 | 1 | 2 | 3 {
  return { front: 0, middle: 1, back: 2, nozzle: 3 }[zone] as 0 | 1 | 2 | 3;
}
