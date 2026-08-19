import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { dryerV1 } from "@/machines/properties";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useDryerMachineIdentification } from "./useDryerMachineIdentification";
import {
  useDryerV1Namespace,
  ScheduleDay,
  SmartTimerEntry,
} from "./dryerV1Namespace";
import {
  DAY_LABELS,
  encodedToTimeInput,
  timeInputToEncoded,
} from "./scheduleEncoding";
import { useMachineMutate } from "@/client/useClient";
import React, { useEffect, useState } from "react";
import { z } from "zod";

const emptySchedule: ScheduleDay[] = Array.from({ length: 7 }, () => ({
  start_time: 0,
  stop_time: 0,
}));

const newEntryDefault: SmartTimerEntry = {
  weekly: true,
  weekday: 0,
  hour_min: 0,
  year: 0,
  month_day: 0,
  enabled: true,
  is_stop: false,
};

/// Weekly start/stop schedule on non-Smart hardware, or a Smart timer-entry table on Smart
/// hardware - gated on `v.is_smart`, reported directly by the connected device. These two
/// UIs are genuinely different (not just a Smart-only section layered on a shared base),
/// unlike Control/Overview/Material.
export function DryerV1SchedulePage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();
  const machineIdentification = useDryerMachineIdentification(
    dryerV1,
    serialString,
  );

  const { liveValues } = useDryerV1Namespace(machineIdentification);
  const v = liveValues?.data;

  const serverSchedule = v?.schedule;
  const [schedule, setSchedule] = useState<ScheduleDay[]>(emptySchedule);

  useEffect(() => {
    if (serverSchedule) setSchedule(serverSchedule);
  }, [serverSchedule]);

  const { request: sendMutation } = useMachineMutate(z.any());

  const updateDay = (index: number, day: ScheduleDay) => {
    const next = schedule.map((d, i) => (i === index ? day : d));
    setSchedule(next);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetSchedule: next },
    });
  };

  const timerEnabled = v?.smart_data.timer_enabled ?? false;
  const entries = v?.smart_data.timer_entries ?? [];

  const handleTimerEnabledChange = (enabled: boolean) => {
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetTimerEnabled: enabled },
    });
  };

  const handleUpdate = (index: number, entry: SmartTimerEntry) => {
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { WriteTimerEntry: { index, entry } },
    });
  };

  const handleDelete = (index: number) => {
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { DeleteTimerEntry: { index } },
    });
  };

  const handleAdd = () => {
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { WriteNewTimerEntry: { entry: newEntryDefault } },
    });
  };

  if (v?.is_smart) {
    return (
      <Page>
        <ControlCard title="Timer Program">
          <SelectionGroupBoolean
            value={timerEnabled}
            optionTrue={{ children: "Enabled", icon: "lu:CalendarClock" }}
            optionFalse={{ children: "Disabled", icon: "lu:CalendarOff" }}
            onChange={handleTimerEnabledChange}
          />
        </ControlCard>

        <ControlCard title="Timer Entries">
          <div className="flex flex-col divide-y divide-gray-100">
            {entries.map((entry, index) => (
              <div
                key={index}
                className="flex flex-wrap items-center gap-4 py-3"
              >
                <SelectionGroupBoolean
                  value={entry.weekly}
                  optionTrue={{ children: "Weekly" }}
                  optionFalse={{ children: "Once" }}
                  onChange={(weekly) =>
                    handleUpdate(index, { ...entry, weekly })
                  }
                />

                {entry.weekly ? (
                  <select
                    className="rounded-lg border border-gray-200 px-2 py-1 text-sm"
                    value={entry.weekday}
                    onChange={(e) =>
                      handleUpdate(index, {
                        ...entry,
                        weekday: Number(e.target.value),
                      })
                    }
                  >
                    {DAY_LABELS.map((label, i) => (
                      <option key={label} value={i}>
                        {label}
                      </option>
                    ))}
                  </select>
                ) : (
                  <label className="flex items-center gap-2 text-sm text-gray-500">
                    Month/Day (MMDD)
                    <input
                      type="number"
                      className="w-24 rounded-lg border border-gray-200 px-2 py-1"
                      value={entry.month_day}
                      onChange={(e) =>
                        handleUpdate(index, {
                          ...entry,
                          month_day: Number(e.target.value),
                        })
                      }
                    />
                  </label>
                )}

                <label className="flex items-center gap-2 text-sm text-gray-500">
                  Time
                  <input
                    type="time"
                    className="rounded-lg border border-gray-200 px-2 py-1"
                    value={encodedToTimeInput(entry.hour_min)}
                    onChange={(e) =>
                      handleUpdate(index, {
                        ...entry,
                        hour_min: timeInputToEncoded(e.target.value),
                      })
                    }
                  />
                </label>

                <SelectionGroup<"Start" | "Stop">
                  value={entry.is_stop ? "Stop" : "Start"}
                  options={{
                    Start: { children: "Start" },
                    Stop: { children: "Stop" },
                  }}
                  onChange={(val) =>
                    handleUpdate(index, { ...entry, is_stop: val === "Stop" })
                  }
                />

                <SelectionGroupBoolean
                  value={entry.enabled}
                  optionTrue={{ children: "On" }}
                  optionFalse={{ children: "Off" }}
                  onChange={(enabled) =>
                    handleUpdate(index, { ...entry, enabled })
                  }
                />

                <TouchButton
                  variant="outline"
                  icon="lu:Trash2"
                  onClick={() => handleDelete(index)}
                >
                  Delete
                </TouchButton>
              </div>
            ))}
          </div>

          <TouchButton icon="lu:Plus" onClick={handleAdd}>
            Add Entry
          </TouchButton>
        </ControlCard>
      </Page>
    );
  }

  return (
    <Page>
      <ControlCard title="Weekly Schedule">
        <div className="flex flex-col divide-y divide-gray-100">
          {DAY_LABELS.map((label, i) => {
            const day = schedule[i] ?? { start_time: 0, stop_time: 0 };
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
                    value={encodedToTimeInput(day.start_time)}
                    onChange={(e) =>
                      updateDay(i, {
                        ...day,
                        start_time: timeInputToEncoded(e.target.value),
                      })
                    }
                  />
                </label>
                <label className="flex items-center gap-2 text-sm text-gray-500">
                  Stop
                  <input
                    type="time"
                    className="rounded-lg border border-gray-200 px-2 py-1"
                    value={encodedToTimeInput(day.stop_time)}
                    onChange={(e) =>
                      updateDay(i, {
                        ...day,
                        stop_time: timeInputToEncoded(e.target.value),
                      })
                    }
                  />
                </label>
                <TouchButton
                  variant="outline"
                  onClick={() => updateDay(i, { start_time: 0, stop_time: 0 })}
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
