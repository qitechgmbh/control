import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useEffect, useState } from "react";
import { ScheduleDay } from "./dryerV1Namespace";
import { useDryerV1 } from "./useDryerV1";

const DAY_LABELS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const STEPS = 48; // 48 x 30 min = 24 h
const TICK_HOURS = [0, 6, 12, 18, 24];

function stepToDisplay(step: number): string {
  const h = Math.floor(step / 2);
  const m = (step % 2) * 30;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

function stepToMinutes(step: number): number {
  return step * 30;
}

function minutesToStep(minutes: number): number {
  return Math.round(minutes / 30);
}

type LocalDay = { enabled: boolean; startStep: number; stopStep: number };

function toLocalDay(day: ScheduleDay): LocalDay {
  const enabled = day.start_minutes !== 0 || day.stop_minutes !== 0;
  return {
    enabled,
    startStep: enabled ? minutesToStep(day.start_minutes) : 16,
    stopStep: enabled ? minutesToStep(day.stop_minutes) : 34,
  };
}

function fromLocalDay(local: LocalDay, autoOn: boolean): ScheduleDay {
  if (!local.enabled || !autoOn) return { start_minutes: 0, stop_minutes: 0 };
  return {
    start_minutes: stepToMinutes(local.startStep),
    stop_minutes: stepToMinutes(local.stopStep),
  };
}

function Toggle({ on, onChange }: { on: boolean; onChange: () => void }) {
  return (
    <button
      onClick={onChange}
      className={`flex h-7 w-14 shrink-0 items-center rounded-full px-1 transition-colors ${
        on ? "bg-green-500" : "bg-gray-300"
      }`}
    >
      <div
        className={`h-5 w-5 rounded-full bg-white shadow transition-transform ${
          on ? "translate-x-7" : "translate-x-0"
        }`}
      />
    </button>
  );
}

function DualRangeSlider({
  startStep,
  stopStep,
  onStart,
  onStop,
  disabled,
}: {
  startStep: number;
  stopStep: number;
  onStart: (v: number) => void;
  onStop: (v: number) => void;
  disabled: boolean;
}) {
  const sp = (startStep / STEPS) * 100;
  const ep = (stopStep / STEPS) * 100;
  const stopOnTop = startStep < STEPS - 1;

  return (
    <div
      className={`flex-1 ${disabled ? "pointer-events-none opacity-30" : ""}`}
    >
      <div className="relative h-3.5">
        {TICK_HOURS.map((h) => (
          <span
            key={h}
            className="absolute -translate-x-1/2 text-[10px] text-gray-400"
            style={{ left: `${(h / 24) * 100}%` }}
          >
            {h === 0 ? "00:00" : h}
          </span>
        ))}
      </div>
      <div className="relative h-6">
        <div className="absolute top-1/2 right-0 left-0 h-1.5 -translate-y-1/2 rounded-full bg-gray-200" />
        <div
          className="absolute top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-green-500"
          style={{ left: `${sp}%`, right: `${100 - ep}%` }}
        />
        <input
          type="range"
          min={0}
          max={STEPS}
          step={1}
          value={startStep}
          onChange={(e) => {
            const v = Number(e.target.value);
            if (v < stopStep) onStart(v);
          }}
          className="dryer-range absolute inset-0 h-full w-full"
          style={{ zIndex: stopOnTop ? 3 : 5 }}
        />
        <input
          type="range"
          min={0}
          max={STEPS}
          step={1}
          value={stopStep}
          onChange={(e) => {
            const v = Number(e.target.value);
            if (v > startStep) onStop(v);
          }}
          className="dryer-range absolute inset-0 h-full w-full"
          style={{ zIndex: stopOnTop ? 5 : 3 }}
        />
      </div>
    </div>
  );
}

export function DryerV1SchedulePage() {
  const { state, setSchedule } = useDryerV1();

  const [days, setDays] = useState<LocalDay[]>(() =>
    Array.from({ length: 7 }, () => ({
      enabled: false,
      startStep: 16,
      stopStep: 34,
    })),
  );
  // isDirty = user has unsaved edits; while dirty we don't let the backend overwrite the UI
  const [isDirty, setIsDirty] = useState(false);
  const [autoOn, setAutoOn] = useState(false);

  useEffect(() => {
    if (state?.schedule && !isDirty) {
      const local = state.schedule.map(toLocalDay);
      setDays(local);
      setAutoOn(local.some((d) => d.enabled));
    }
  }, [state?.schedule, isDirty]);

  const handleSave = () => {
    setSchedule(days.map((d) => fromLocalDay(d, autoOn)));
    setIsDirty(false);
  };

  const setDay = (i: number, patch: Partial<LocalDay>) => {
    setIsDirty(true);
    setDays((prev) =>
      prev.map((d, idx) => (idx === i ? { ...d, ...patch } : d)),
    );
  };

  return (
    <Page className="h-full">
      <style>{`
        .dryer-range {
          -webkit-appearance: none;
          appearance: none;
          background: transparent;
          cursor: pointer;
          pointer-events: none;
        }
        .dryer-range::-webkit-slider-runnable-track {
          background: transparent;
        }
        .dryer-range::-webkit-slider-thumb {
          -webkit-appearance: none;
          pointer-events: all;
          height: 20px;
          width: 20px;
          border-radius: 50%;
          background: white;
          border: 1.5px solid #d1d5db;
          box-shadow: 0 1px 4px rgba(0,0,0,0.2);
          cursor: grab;
          margin-top: -2px;
        }
        .dryer-range:active::-webkit-slider-thumb {
          cursor: grabbing;
          border-color: #22c55e;
        }
      `}</style>

      <div className="flex w-full flex-1 flex-col rounded-2xl border border-gray-200 bg-white p-6 shadow-sm">
        <div className="mb-6 flex items-center justify-between">
          <h2 className="text-lg font-bold text-gray-800">
            Machine Operating Schedule
          </h2>
          <div className="flex items-center gap-2.5">
            <Toggle
              on={autoOn}
              onChange={() => {
                setIsDirty(true);
                setAutoOn((v) => !v);
              }}
            />
            <span
              className={`text-sm font-semibold ${autoOn ? "text-green-600" : "text-gray-400"}`}
            >
              Auto Schedule: {autoOn ? "On" : "Off"}
            </span>
          </div>
        </div>

        <div
          className={`flex flex-1 flex-col justify-between ${!autoOn ? "pointer-events-none opacity-40" : ""}`}
        >
          {days.map((day, i) => (
            <div key={i} className="flex items-center gap-3">
              <span className="w-8 shrink-0 text-sm font-bold text-gray-700">
                {DAY_LABELS[i]}
              </span>
              <Toggle
                on={day.enabled}
                onChange={() => setDay(i, { enabled: !day.enabled })}
              />
              <span className="w-6 shrink-0 text-xs font-semibold text-gray-500">
                {day.enabled ? "On" : "Off"}
              </span>
              <DualRangeSlider
                startStep={day.startStep}
                stopStep={day.stopStep}
                onStart={(val) => setDay(i, { startStep: val })}
                onStop={(val) => setDay(i, { stopStep: val })}
                disabled={!day.enabled}
              />
              <span className="w-32 shrink-0 text-right text-sm text-gray-500">
                {day.enabled
                  ? `${stepToDisplay(day.startStep)} – ${stepToDisplay(day.stopStep)}`
                  : "—"}
              </span>
            </div>
          ))}
        </div>

        <div className="mt-6 flex justify-end border-t border-gray-100 pt-4">
          <TouchButton icon="lu:Save" onClick={handleSave}>
            Save
          </TouchButton>
        </div>
      </div>
    </Page>
  );
}
