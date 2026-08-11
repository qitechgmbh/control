import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React, { useEffect, useState } from "react";
import {
  calculateCalibration,
  CalibrationRole,
  getExtruderMixerConfig,
  saveExtruderMixerConfig,
  setActiveMixerCalibration,
  useExtruderMixerStorage,
} from "./extruderMixerConfig";

const roleDetails: Record<
  CalibrationRole,
  { title: string; description: string; defaultName: string }
> = {
  hopperA: {
    title: "Hopper A Feeder",
    description: "Collect material discharged by the left dosing auger.",
    defaultName: "Material A",
  },
  hopperB: {
    title: "Hopper B Feeder",
    description: "Collect material discharged by the right dosing auger.",
    defaultName: "Material B",
  },
};

function CalibrationCard({
  role,
  className,
  activeCalibration,
  setActiveCalibration,
}: {
  role: CalibrationRole;
  className?: string;
  activeCalibration: CalibrationRole | null;
  setActiveCalibration: (role: CalibrationRole | null) => void;
}) {
  const { config } = useExtruderMixerStorage();
  const saved = config.calibration[role];
  const details = roleDetails[role];
  const [name, setName] = useState(saved?.name ?? details.defaultName);
  const [testRpm, setTestRpm] = useState(saved?.testRpm ?? 20);
  const [durationSeconds, setDurationSeconds] = useState(
    saved?.durationSeconds ?? 60,
  );
  const [collectedGrams, setCollectedGrams] = useState(
    saved?.collectedGrams ?? 0,
  );
  const [remainingSeconds, setRemainingSeconds] = useState(0);
  const running = activeCalibration === role;

  useEffect(() => {
    if (!saved) return;
    setName(saved.name);
    setTestRpm(saved.testRpm);
    setDurationSeconds(saved.durationSeconds);
    setCollectedGrams(saved.collectedGrams);
  }, [saved]);

  useEffect(() => {
    if (!running) {
      setRemainingSeconds(0);
      return;
    }

    setRemainingSeconds(durationSeconds);
    const interval = window.setInterval(() => {
      setRemainingSeconds((remaining) => {
        if (remaining <= 1) {
          window.clearInterval(interval);
          setActiveCalibration(null);
          return 0;
        }
        return remaining - 1;
      });
    }, 1000);

    return () => window.clearInterval(interval);
  }, [durationSeconds, role, running, setActiveCalibration]);

  const valid = testRpm > 0 && durationSeconds > 0 && collectedGrams > 0;
  const testSettingsValid = testRpm > 0 && durationSeconds > 0;
  const preview = valid
    ? calculateCalibration(name, testRpm, durationSeconds, collectedGrams)
    : null;

  const save = () => {
    if (!preview) return;
    const current = getExtruderMixerConfig();
    saveExtruderMixerConfig({
      ...current,
      calibration: { ...current.calibration, [role]: preview },
    });
  };

  return (
    <ControlCard title={details.title} className={className}>
      <p className="text-sm text-gray-500">{details.description}</p>
      <div className="grid grid-cols-2 gap-3">
        <Label label="Profile Name">
          <input
            className="h-12 w-full rounded-md border border-gray-200 px-3"
            disabled={activeCalibration !== null}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        </Label>
        <Label label="Test RPM">
          <input
            className="h-12 w-full rounded-md border border-gray-200 px-3"
            disabled={activeCalibration !== null}
            type="number"
            min="0.1"
            step="0.1"
            value={testRpm}
            onChange={(event) => setTestRpm(Number(event.target.value))}
          />
        </Label>
        <Label label="Run Duration">
          <div className="relative">
            <input
              className="h-12 w-full rounded-md border border-gray-200 px-3 pr-12"
              disabled={activeCalibration !== null}
              type="number"
              min="1"
              value={durationSeconds}
              onChange={(event) =>
                setDurationSeconds(Number(event.target.value))
              }
            />
            <span className="absolute top-3 right-3 text-sm text-gray-500">
              sec
            </span>
          </div>
        </Label>
        <Label label="Collected Weight">
          <div className="relative">
            <input
              className="h-12 w-full rounded-md border border-gray-200 px-3 pr-10"
              type="number"
              min="0"
              step="0.1"
              value={collectedGrams}
              onChange={(event) =>
                setCollectedGrams(Number(event.target.value))
              }
            />
            <span className="absolute top-3 right-3 text-sm text-gray-500">
              g
            </span>
          </div>
        </Label>
      </div>
      <div className="flex items-center gap-3">
        {running ? (
          <TouchButton
            variant="destructive"
            icon="lu:Square"
            onClick={() => setActiveCalibration(null)}
          >
            Stop Test · {remainingSeconds}s
          </TouchButton>
        ) : (
          <TouchButton
            icon="lu:Play"
            disabled={activeCalibration !== null || !testSettingsValid}
            onClick={() => setActiveCalibration(role)}
          >
            Run Auger
          </TouchButton>
        )}
        {activeCalibration !== null && !running && (
          <span className="text-sm text-gray-500">Other auger is running</span>
        )}
      </div>
      <div className="rounded-xl border border-gray-200 bg-gray-50 p-4 text-sm">
        <span className="text-gray-500">Calculated output</span>
        <strong className="mt-1 block text-xl">
          {preview ? `${preview.rateKgPerHour.toFixed(3)} kg/h` : "—"}
        </strong>
        <span className="text-xs text-gray-500">
          {preview
            ? `${preview.kgPerHourPerRpm.toFixed(5)} kg/h per rpm`
            : "Enter RPM, duration, and measured weight."}
        </span>
      </div>
      <TouchButton
        icon="lu:Save"
        disabled={!valid || activeCalibration !== null}
        onClick={save}
      >
        Save Calibration
      </TouchButton>
    </ControlCard>
  );
}

export function ExtruderMixerSettingsPage() {
  const { config, activeCalibration } = useExtruderMixerStorage();

  const setDirection = (
    motor: keyof typeof config.motorForward,
    forward: boolean,
  ) => {
    const current = getExtruderMixerConfig();
    saveExtruderMixerConfig({
      ...current,
      motorForward: { ...current.motorForward, [motor]: forward },
    });
  };

  return (
    <Page>
      <Alert className="border-amber-300 bg-amber-50">
        <AlertTitle>Calibration requires a physical collection test</AlertTitle>
        <AlertDescription>
          Run one side auger at a known RPM for a measured time, collect its
          output, weigh it, then enter the result below. Stop the machine before
          changing motor direction.
        </AlertDescription>
      </Alert>

      <ControlCard title="Motor Direction">
        <ControlGrid columns={2}>
          {[
            { key: "hopperA" as const, label: "Hopper A Feeder" },
            { key: "hopperB" as const, label: "Hopper B Feeder" },
          ].map((motor) => (
            <Label key={motor.key} label={motor.label}>
              <SelectionGroupBoolean
                value={config.motorForward[motor.key]}
                disabled={activeCalibration !== null}
                optionTrue={{ children: "Forward", icon: "lu:RotateCw" }}
                optionFalse={{ children: "Reverse", icon: "lu:RotateCcw" }}
                onChange={(forward) => setDirection(motor.key, forward)}
              />
            </Label>
          ))}
        </ControlGrid>
      </ControlCard>

      <ControlGrid>
        <CalibrationCard
          role="hopperA"
          activeCalibration={activeCalibration}
          setActiveCalibration={setActiveMixerCalibration}
        />
        <CalibrationCard
          role="hopperB"
          className="xl:col-start-3"
          activeCalibration={activeCalibration}
          setActiveCalibration={setActiveMixerCalibration}
        />
      </ControlGrid>
    </Page>
  );
}
