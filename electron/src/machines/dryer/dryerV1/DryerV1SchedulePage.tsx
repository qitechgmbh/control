import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useEffect, useState } from "react";
import { ScheduleDay } from "./dryerV1Namespace";
import { useDryerV1 } from "./useDryerV1";

const DAY_LABELS = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
];

const emptySchedule: ScheduleDay[] = Array.from({ length: 7 }, () => ({
  start_minutes: 0,
  stop_minutes: 0,
}));

/// Converts to the value format expected by `<input type="time">`.
function minutesToTimeInput(minutes: number): string {
  if (!minutes) return "";
  const hh = Math.floor(minutes / 60);
  const mm = minutes % 60;
  return `${String(hh).padStart(2, "0")}:${String(mm).padStart(2, "0")}`;
}

/// Converts back from an `<input type="time">` value ("HH:MM").
function timeInputToMinutes(value: string): number {
  if (!value) return 0;
  const [hh, mm] = value.split(":").map(Number);
  return hh * 60 + mm;
}

export function DryerV1SchedulePage() {
  const { state, setSchedule } = useDryerV1();

  const [schedule, setLocalSchedule] = useState<ScheduleDay[]>(emptySchedule);

  useEffect(() => {
    if (state?.schedule) setLocalSchedule(state.schedule);
  }, [state?.schedule]);

  const updateDay = (index: number, day: ScheduleDay) => {
    const next = schedule.map((d, i) => (i === index ? day : d));
    setLocalSchedule(next);
    setSchedule(next);
  };

  return (
    <Page>
      <ControlCard title="Weekly Schedule">
        <p className="text-sm text-gray-500">
          Set a start/stop time per day. A day with no stop time uses the
          drying timer instead (see Control tab).
        </p>
        <div className="flex flex-col divide-y divide-gray-100">
          {DAY_LABELS.map((label, i) => {
            const day = schedule[i] ?? { start_minutes: 0, stop_minutes: 0 };
            return (
              <div
                key={label}
                className="flex flex-wrap items-center gap-4 py-3"
              >
                <span className="w-28 shrink-0 font-semibold text-gray-700">
                  {label}
                </span>
                <label className="flex items-center gap-2 text-sm text-gray-500">
                  Start
                  <input
                    type="time"
                    className="rounded-lg border border-gray-200 px-2 py-1"
                    value={minutesToTimeInput(day.start_minutes)}
                    onChange={(e) =>
                      updateDay(i, {
                        ...day,
                        start_minutes: timeInputToMinutes(e.target.value),
                      })
                    }
                  />
                </label>
                <label className="flex items-center gap-2 text-sm text-gray-500">
                  Stop
                  <input
                    type="time"
                    className="rounded-lg border border-gray-200 px-2 py-1"
                    value={minutesToTimeInput(day.stop_minutes)}
                    onChange={(e) =>
                      updateDay(i, {
                        ...day,
                        stop_minutes: timeInputToMinutes(e.target.value),
                      })
                    }
                  />
                </label>
                <TouchButton
                  variant="outline"
                  onClick={() =>
                    updateDay(i, { start_minutes: 0, stop_minutes: 0 })
                  }
                >
                  Clear
                </TouchButton>
              </div>
            );
          })}
        </div>
      </ControlCard>
    </Page>
  );
}
