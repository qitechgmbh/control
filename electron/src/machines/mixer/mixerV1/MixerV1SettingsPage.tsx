import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React, { useEffect, useState } from "react";
import { useMixerV1 } from "./useMixerV1";

const MOTOR_FULL_STEPS_PER_REV = 200;

function calculateCalibration(
  testRpm: number,
  durationSeconds: number,
  collectedGrams: number,
) {
  const rateKgPerHour = (collectedGrams * 3.6) / durationSeconds;
  const stepsPerSecondAtTestRpm = (testRpm * MOTOR_FULL_STEPS_PER_REV) / 60;
  const calibrationStepsPerKgh = stepsPerSecondAtTestRpm / rateKgPerHour;
  return { rateKgPerHour, calibrationStepsPerKgh };
}

type CalibrationCardProps = {
  title: string;
  description: string;
  className?: string;
  currentCalibration: number;
  running: boolean;
  disabled: boolean;
  setEnabled: (enabled: boolean) => void;
  setTargetRpm: (rpm: number) => void;
  saveCalibration: (value: number) => void;
  setRunning: (running: boolean) => void;
};

function CalibrationCard({
  title,
  description,
  className,
  currentCalibration,
  running,
  disabled,
  setEnabled,
  setTargetRpm,
  saveCalibration,
  setRunning,
}: CalibrationCardProps) {
  const [testRpm, setTestRpm] = useState(20);
  const [durationSeconds, setDurationSeconds] = useState(60);
  const [collectedGrams, setCollectedGrams] = useState(0);
  const [remainingSeconds, setRemainingSeconds] = useState(0);

  useEffect(() => {
    if (!running) {
      setRemainingSeconds(0);
      return;
    }

    setTargetRpm(testRpm);
    setEnabled(true);
    setRemainingSeconds(durationSeconds);

    const interval = window.setInterval(() => {
      setRemainingSeconds((remaining) => {
        if (remaining <= 1) {
          window.clearInterval(interval);
          setEnabled(false);
          setTargetRpm(0);
          setRunning(false);
          return 0;
        }
        return remaining - 1;
      });
    }, 1000);

    return () => window.clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [running]);

  const stop = () => {
    setEnabled(false);
    setTargetRpm(0);
    setRunning(false);
  };

  const valid = testRpm > 0 && durationSeconds > 0 && collectedGrams > 0;
  const testSettingsValid = testRpm > 0 && durationSeconds > 0;
  const preview = valid
    ? calculateCalibration(testRpm, durationSeconds, collectedGrams)
    : null;

  return (
    <ControlCard title={title} className={className}>
      <p className="text-sm text-gray-500">{description}</p>
      <div className="grid grid-cols-2 gap-3">
        <Label label="Test RPM">
          <input
            className="h-12 w-full rounded-md border border-gray-200 px-3"
            disabled={running}
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
              disabled={running}
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
          <TouchButton variant="destructive" icon="lu:Square" onClick={stop}>
            Stop Test · {remainingSeconds}s
          </TouchButton>
        ) : (
          <TouchButton
            icon="lu:Play"
            disabled={disabled || !testSettingsValid}
            onClick={() => setRunning(true)}
          >
            Run Auger
          </TouchButton>
        )}
        {disabled && !running && (
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
            ? `${preview.calibrationStepsPerKgh.toFixed(3)} steps/sec per kg/h`
            : "Enter RPM, duration, and measured weight."}
        </span>
        <span className="mt-2 block text-xs text-gray-400">
          Current saved value: {currentCalibration.toFixed(3)} steps/sec per
          kg/h
        </span>
      </div>
      <TouchButton
        icon="lu:Save"
        disabled={!valid || running}
        onClick={() => preview && saveCalibration(preview.calibrationStepsPerKgh)}
      >
        Save Calibration
      </TouchButton>
    </ControlCard>
  );
}

export function MixerV1SettingsPage() {
  const {
    state,
    setHopperAForward,
    setHopperAEnabled,
    setHopperATargetRpm,
    setHopperACalibrationStepsPerKgh,
    setHopperBForward,
    setHopperBEnabled,
    setHopperBTargetRpm,
    setHopperBCalibrationStepsPerKgh,
  } = useMixerV1();

  const [activeCalibration, setActiveCalibration] = useState<
    "hopperA" | "hopperB" | null
  >(null);

  return (
    <Page>
      <Alert className="border-amber-300 bg-amber-50">
        <AlertTitle>Calibration requires a physical collection test</AlertTitle>
        <AlertDescription>
          Run one side auger at a known RPM for a measured time, collect its
          output, weigh it, then enter the result below. Stop the machine
          before changing motor direction.
        </AlertDescription>
      </Alert>

      <ControlCard title="Motor Direction">
        <ControlGrid columns={2}>
          <Label label="Hopper A Feeder">
            <SelectionGroupBoolean
              value={state?.hopper_a_state.forward}
              disabled={activeCalibration !== null}
              optionTrue={{ children: "Forward", icon: "lu:RotateCw" }}
              optionFalse={{ children: "Reverse", icon: "lu:RotateCcw" }}
              onChange={setHopperAForward}
            />
          </Label>
          <Label label="Hopper B Feeder">
            <SelectionGroupBoolean
              value={state?.hopper_b_state.forward}
              disabled={activeCalibration !== null}
              optionTrue={{ children: "Forward", icon: "lu:RotateCw" }}
              optionFalse={{ children: "Reverse", icon: "lu:RotateCcw" }}
              onChange={setHopperBForward}
            />
          </Label>
        </ControlGrid>
      </ControlCard>

      <ControlGrid>
        <CalibrationCard
          title="Hopper A Feeder"
          description="Collect material discharged by the left dosing auger."
          currentCalibration={
            state?.hopper_a_state.calibration_steps_per_kgh ?? 0
          }
          running={activeCalibration === "hopperA"}
          disabled={activeCalibration !== null && activeCalibration !== "hopperA"}
          setEnabled={setHopperAEnabled}
          setTargetRpm={setHopperATargetRpm}
          saveCalibration={setHopperACalibrationStepsPerKgh}
          setRunning={(running) =>
            setActiveCalibration(running ? "hopperA" : null)
          }
        />
        <CalibrationCard
          title="Hopper B Feeder"
          className="xl:col-start-3"
          description="Collect material discharged by the right dosing auger."
          currentCalibration={
            state?.hopper_b_state.calibration_steps_per_kgh ?? 0
          }
          running={activeCalibration === "hopperB"}
          disabled={activeCalibration !== null && activeCalibration !== "hopperB"}
          setEnabled={setHopperBEnabled}
          setTargetRpm={setHopperBTargetRpm}
          saveCalibration={setHopperBCalibrationStepsPerKgh}
          setRunning={(running) =>
            setActiveCalibration(running ? "hopperB" : null)
          }
        />
      </ControlGrid>
    </Page>
  );
}
