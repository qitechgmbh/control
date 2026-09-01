import { ScheduleDay } from "./dryerNamespace";

/// Helpers for the dryer's `HH*100+MM` time encoding (e.g. 1430 = 14:30), shared by the
/// Control page (countdown/schedule-stop display) and the Schedule page (weekly
/// start/stop, Smart timer entries).

export const DAY_LABELS = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
];

export function encodedToMinutes(encoded: number): number {
  return Math.floor(encoded / 100) * 60 + (encoded % 100);
}

/// Today's weekly schedule entry (JS `Date.getDay()` is 0=Sun, `schedule` is 0=Mon).
export function getTodayScheduleDay(
  schedule: ScheduleDay[] | undefined,
): ScheduleDay | undefined {
  if (!schedule) return undefined;
  const idx = (new Date().getDay() + 6) % 7;
  return schedule[idx];
}

export function formatMinutes(totalMins: number): string {
  const h = Math.floor(totalMins / 60);
  const m = totalMins % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

/// Converts to the value format expected by `<input type="time">`.
export function encodedToTimeInput(encoded: number): string {
  if (!encoded) return "";
  const hh = Math.floor(encoded / 100);
  const mm = encoded % 100;
  return `${String(hh).padStart(2, "0")}:${String(mm).padStart(2, "0")}`;
}

/// Converts back from an `<input type="time">` value ("HH:MM").
export function timeInputToEncoded(value: string): number {
  if (!value) return 0;
  const [hh, mm] = value.split(":").map(Number);
  return hh * 100 + mm;
}
