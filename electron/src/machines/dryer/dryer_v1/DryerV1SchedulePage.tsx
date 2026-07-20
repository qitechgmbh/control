import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { dryerV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { dryerV1SerialRoute } from "@/routes/routes";
import { useDryerV1Namespace, ScheduleDay } from "./dryerV1Namespace";
import { useMachineMutate } from "@/client/useClient";
import React, { useEffect, useMemo, useState } from "react";
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

const emptySchedule: ScheduleDay[] = Array.from({ length: 7 }, () => ({
  start_time: 0,
  stop_time: 0,
}));

export function DryerV1SchedulePage() {
  const { serial: serialString } = dryerV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: dryerV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const { liveValues } = useDryerV1Namespace(machineIdentification);
  const serverSchedule = liveValues?.data.schedule;

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
