import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import type { SafetyMessageState } from "../state/gluetexNamespace";

function decodeHeaterZones(zoneMask: number): number[] {
  const zones: number[] = [];
  for (let i = 0; i < 6; i++) {
    if (zoneMask & (1 << i)) zones.push(i + 1);
  }
  return zones;
}

type ReasonKind =
  | "WinderTensionArm"
  | "TapeFeederTensionArm"
  | "InletTensionArm"
  | "Optris1Voltage"
  | "Optris2Voltage"
  | "HeaterOverTemperature"
  | "Bandueberwachung"
  | "SleepTimer"
  | "Unknown";

function reasonKind(reason: unknown): ReasonKind {
  if (typeof reason === "string") {
    switch (reason) {
      case "WinderTensionArm":
      case "TapeFeederTensionArm":
      case "InletTensionArm":
      case "Optris1Voltage":
      case "Optris2Voltage":
      case "Bandueberwachung":
      case "SleepTimer":
        return reason;
      default:
        return "Unknown";
    }
  }
  if (typeof reason === "object" && reason !== null && "HeaterOverTemperature" in reason) {
    return "HeaterOverTemperature";
  }
  return "Unknown";
}

function reasonTitle(kind: ReasonKind): string {
  switch (kind) {
    case "WinderTensionArm":
      return "Winder Tension Arm Out of Range";
    case "TapeFeederTensionArm":
      return "TA Tape Feeder Out of Range";
    case "InletTensionArm":
      return "TA Inlet Feeder Out of Range";
    case "Optris1Voltage":
      return "Optris 1 Voltage Out of Range";
    case "Optris2Voltage":
      return "Optris 2 Voltage Out of Range";
    case "HeaterOverTemperature":
      return "Heater Over-Temperature";
    case "Bandueberwachung":
      return "Band Monitoring Alert";
    case "SleepTimer":
      return "Sleep Timer";
    default:
      return "Unexpected Safety Condition";
  }
}

function reasonDescription(kind: ReasonKind): string {
  switch (kind) {
    case "WinderTensionArm":
    case "TapeFeederTensionArm":
    case "InletTensionArm":
      return "This tension arm exceeded its configured safety limits. Motors were automatically stopped to prevent damage.";
    case "Optris1Voltage":
    case "Optris2Voltage":
      return "This Optris voltage sensor detected a reading outside the configured safe range. Motors were automatically stopped.";
    case "HeaterOverTemperature":
      return "One or more heater zones exceeded their configured temperature limit. Motors and heaters were automatically shut down. This will reappear immediately if the zone is still over temperature.";
    case "Bandueberwachung":
      return "The band monitoring system detected that the band is absent or broken. Motors were automatically stopped.";
    case "SleepTimer":
      return "The machine automatically entered standby mode due to inactivity in Setup mode.";
    default:
      return "An unexpected safety condition triggered an automatic stop. Check the machine before resuming.";
  }
}

function SafetyMessageCard({
  message,
  firstSeenAt,
  onAcknowledge,
}: {
  message: SafetyMessageState;
  firstSeenAt: number;
  onAcknowledge: () => void;
}) {
  const kind = reasonKind(message.reason);
  const heaterZones =
    kind === "HeaterOverTemperature" &&
    typeof message.reason === "object" &&
    message.reason !== null &&
    "HeaterOverTemperature" in message.reason
      ? decodeHeaterZones(
          (message.reason as { HeaterOverTemperature: { zones: number } })
            .HeaterOverTemperature.zones,
        )
      : [];

  // Computed from a locally-tracked first-seen timestamp (see
  // GluetexSafetyInbox below), not directly from message.age_ms — the
  // backend only recomputes age_ms when it happens to emit a StateEvent for
  // some other reason, so a snapshot value would otherwise appear frozen
  // between emissions instead of ticking up live.
  const ageSeconds = Math.max(0, Math.floor((Date.now() - firstSeenAt) / 1000));

  return (
    <div className="space-y-2 rounded-md border border-red-500/40 bg-red-500/5 p-3">
      <div className="flex items-start justify-between gap-2">
        <div>
          <p className="font-semibold text-red-600 dark:text-red-400">
            {reasonTitle(kind)}
          </p>
          <p className="text-sm text-red-600/80 dark:text-red-400/80">
            {reasonDescription(kind)}
          </p>
          {kind === "HeaterOverTemperature" && heaterZones.length > 0 && (
            <p className="mt-1 text-sm text-red-600/90 dark:text-red-400/90">
              Zones: {heaterZones.join(", ")}
            </p>
          )}
        </div>
        <span className="shrink-0 rounded-full bg-red-100 px-2 py-0.5 text-xs font-semibold text-red-700 dark:bg-red-950 dark:text-red-300">
          {message.severity === "Full" ? "Motors + Heaters" : "Motors"}
        </span>
      </div>
      <p className="text-xs text-red-600/70 dark:text-red-400/70">
        First seen {ageSeconds}s ago
        {message.occurrence_count > 1
          ? ` · recurred ${message.occurrence_count}x`
          : ""}
      </p>
      <TouchButton
        variant="destructive"
        className="w-full"
        onClick={onAcknowledge}
      >
        Acknowledge
      </TouchButton>
    </div>
  );
}

export function GluetexSafetyInbox() {
  const { state, acknowledgeSafetyMessage, acknowledgeAllSafetyMessages } =
    useGluetex();

  const pendingSafetyMessages = state?.pending_safety_messages ?? [];

  // Re-render once a second so each card's "Xs ago" ticks up live, instead
  // of only updating whenever the backend happens to emit a new StateEvent.
  const [, forceTick] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => {
    const interval = setInterval(forceTick, 1000);
    return () => clearInterval(interval);
  }, []);

  // Track when this client first observed each message, seeded from the
  // backend's age_ms so a client that reconnects mid-incident still shows
  // the correct elapsed time immediately rather than starting over at 0.
  const firstSeenRef = React.useRef<Map<number, number>>(new Map());
  const activeIds = new Set(pendingSafetyMessages.map((m) => m.id));
  for (const message of pendingSafetyMessages) {
    if (!firstSeenRef.current.has(message.id)) {
      firstSeenRef.current.set(message.id, Date.now() - message.age_ms);
    }
  }
  for (const id of firstSeenRef.current.keys()) {
    if (!activeIds.has(id)) firstSeenRef.current.delete(id);
  }

  if (pendingSafetyMessages.length === 0) return null;

  return (
    <Dialog open onOpenChange={() => {}}>
      <DialogContent
        className="flex max-h-[80vh] max-w-2xl flex-col overflow-hidden border-red-500"
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <div className="flex items-center gap-3">
            <span className="text-4xl">🛑</span>
            <DialogTitle className="text-2xl font-bold text-red-600 dark:text-red-400">
              SAFETY STOP
              {pendingSafetyMessages.length > 1
                ? `S (${pendingSafetyMessages.length})`
                : ""}
            </DialogTitle>
          </div>
          <DialogDescription className="sr-only">
            {pendingSafetyMessages.length} pending safety message
            {pendingSafetyMessages.length > 1 ? "s" : ""} require acknowledgement
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 space-y-3 overflow-y-auto">
          {pendingSafetyMessages.map((message) => (
            <SafetyMessageCard
              key={message.id}
              message={message}
              firstSeenAt={
                firstSeenRef.current.get(message.id) ?? Date.now()
              }
              onAcknowledge={() => acknowledgeSafetyMessage(message.id)}
            />
          ))}
        </div>

        {pendingSafetyMessages.length > 1 && (
          <TouchButton
            variant="destructive"
            className="mt-2 w-full"
            onClick={acknowledgeAllSafetyMessages}
          >
            Acknowledge All ({pendingSafetyMessages.length})
          </TouchButton>
        )}
      </DialogContent>
    </Dialog>
  );
}
