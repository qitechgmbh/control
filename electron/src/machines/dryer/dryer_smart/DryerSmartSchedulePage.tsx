import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { SelectionGroup, SelectionGroupBoolean } from "@/control/SelectionGroup";
import { dryerSmart } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerSmartSerialRoute } from "@/routes/routes";
import {
  useDryerSmartNamespace,
  SmartTimerEntry,
} from "./dryerSmartNamespace";
import { useMachineMutate } from "@/client/useClient";
import React, { useMemo } from "react";
import { z } from "zod";

const DAY_LABELS = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
];

function encodedToTimeInput(encoded: number): string {
  if (!encoded) return "";
  const hh = Math.floor(encoded / 100);
  const mm = encoded % 100;
  return `${String(hh).padStart(2, "0")}:${String(mm).padStart(2, "0")}`;
}

function timeInputToEncoded(value: string): number {
  if (!value) return 0;
  const [hh, mm] = value.split(":").map(Number);
  return hh * 100 + mm;
}

const newEntryDefault: SmartTimerEntry = {
  weekly: true,
  weekday: 0,
  hour_min: 0,
  year: 0,
  month_day: 0,
  enabled: true,
  is_stop: false,
};

export function DryerSmartSchedulePage() {
  const { serial: serialString } = dryerSmartSerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerSmart.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const { liveValues } = useDryerSmartNamespace(machineIdentification);
  const timerEnabled = liveValues?.data.smart_data.timer_enabled ?? false;
  const entries = liveValues?.data.smart_data.timer_entries ?? [];

  const { request: sendMutation } = useMachineMutate(z.any());

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
