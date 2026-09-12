import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { SelectionGroup } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { Icon } from "@/components/Icon";
import React, { useMemo } from "react";
import { useDryerV1 } from "./useDryerV1";

function isRunningStatus(status: number): boolean {
  return status !== 1 && status !== 5 && status !== 6;
}

type StatusInfo = {
  label: string;
  icon: string;
};

function getMachineStatusInfo(status: number): StatusInfo {
  switch (status) {
    case 0:
      return { label: "Starting", icon: "lu:Power" };
    case 1:
      return { label: "Standby", icon: "lu:CirclePause" };
    case 2:
      return { label: "Heating", icon: "lu:Flame" };
    case 3:
      return { label: "Starting Up", icon: "lu:Settings" };
    case 4:
      return { label: "Drying", icon: "lu:Wind" };
    case 5:
      return { label: "Cooling", icon: "lu:Snowflake" };
    case 6:
      return { label: "Fan Stop", icon: "lu:Fan" };
    case 7:
      return { label: "Finishing", icon: "lu:Snowflake" };
    case 9:
      return { label: "Switching", icon: "lu:RefreshCw" };
    default:
      return { label: `Status ${status}`, icon: "lu:HelpCircle" };
  }
}

function getAlarmMessage(code: number): string | null {
  switch (code) {
    case 0:
      return null;
    case 1:
      return "Local control active";
    case 2:
      return "Keyboard error";
    case 3:
      return "Sensor break T1";
    case 4:
      return "Sensor break T2";
    case 5:
      return "Sensor break T3";
    case 6:
      return "Sensor break T4";
    case 7:
      return "Sensor break T5";
    case 8:
      return "Sensor break T6";
    case 9:
      return "Process temp exceeded";
    case 11:
      return "Thermal protection regen";
    case 12:
      return "Thermal protection fan";
    case 14:
      return "Fan over-temperature";
    case 15:
      return "Check process fan";
    default:
      return `Alarm ${code}`;
  }
}

function getWarningMessage(code: number): string | null {
  switch (code) {
    case 0:
      return null;
    case 1:
      return "Warning 36";
    case 2:
      return "Filter cleaning required (pressure)";
    case 3:
      return "Filter cleaning required (time)";
    case 4:
      return "High dew point";
    case 5:
      return "Low process temperature";
    case 6:
      return "Slave dryer warning";
    case 7:
      return "MPM temperature reduced";
    case 8:
      return "MPM dryer in standby";
    default:
      return `Warning ${code}`;
  }
}

function formatRemaining(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}min ${s}s`;
}

function formatMinutes(totalMins: number): string {
  const h = Math.floor(totalMins / 60);
  const m = totalMins % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

export function DryerV1ControlPage() {
  const {
    state,
    liveValues,
    ts_temp_process,
    ts_temp_regen_in,
    ts_temp_regen_out,
    ts_temp_fan_inlet,
    ts_temp_return_air,
    ts_pwm_fan1,
    ts_pwm_fan2,
    ts_power_process,
    ts_power_regen,
    setRunning,
    setTargetTemperature,
    setAirVolume,
    setDryingTimerMinutes,
  } = useDryerV1();

  const isRunning = !!state && isRunningStatus(state.status);
  const statusInfo = state ? getMachineStatusInfo(state.status) : null;
  const alarmMessage = state ? getAlarmMessage(state.alarm) : null;
  const warningMessage = state ? getWarningMessage(state.warning) : null;
  const remainingSec = liveValues?.remaining_seconds ?? null;

  // Today's scheduled stop, if any - the backend already enforces this server-side
  // (see check_auto_stop/compute_remaining_seconds), this is purely a display hint
  // for which of Schedule vs. Drying Timer is currently in control.
  const scheduledStopMins = useMemo(() => {
    if (!state?.schedule) return null;
    const idx = (new Date().getDay() + 6) % 7; // JS 0=Sun -> schedule's 0=Mon
    const today = state.schedule[idx];
    if (!today?.stop_minutes) return null;
    const nowMins = new Date().getHours() * 60 + new Date().getMinutes();
    return today.stop_minutes > nowMins ? today.stop_minutes : null;
  }, [state?.schedule]);
  const scheduleControlled = scheduledStopMins !== null;

  return (
    <Page>
      <ControlGrid columns={3}>
        <ControlCard title="Timer">
          <div className="flex items-center gap-1.5 text-sm text-gray-400">
            <Icon name="lu:Clock" className="size-4" />
            <span>Remaining Time</span>
          </div>
          <div className="text-3xl font-bold">
            {remainingSec !== null ? formatRemaining(remainingSec) : "—"}
          </div>
          {scheduleControlled ? (
            <div className="flex items-center gap-1.5 rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-sm font-semibold text-blue-700">
              <Icon name="lu:CalendarClock" className="size-4" />
              <span>Schedule: stops at {formatMinutes(scheduledStopMins!)}</span>
            </div>
          ) : (
            <Label label="Drying Timer">
              <EditValue
                title="Drying Timer"
                value={state?.drying_timer_minutes}
                min={1}
                max={1440}
                step={5}
                disabled={state === null}
                renderValue={(val) => `${val.toFixed(0)} min`}
                onChange={setDryingTimerMinutes}
              />
            </Label>
          )}
        </ControlCard>

        <ControlCard title="Temperature">
          <TimeSeriesValueNumeric
            label="Current Temperature"
            unit="C"
            timeseries={ts_temp_process}
            renderValue={(val) => val.toFixed(1)}
          />
          <Label label="Target Temperature">
            <EditValue
              title="Target Temperature"
              value={state?.target_temperature}
              min={50}
              max={180}
              step={1}
              disabled={state === null}
              renderValue={(val) => val.toFixed(0)}
              unit="C"
              onChange={setTargetTemperature}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Air Volume">
          <p className="text-xs text-gray-400">
            Manual setpoint, raw device unit.
          </p>
          <Label label="Air Volume">
            <EditValue
              title="Air Volume"
              value={state?.air_volume}
              min={1}
              step={1}
              disabled={state === null}
              renderValue={(val) => val.toFixed(0)}
              onChange={setAirVolume}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Mode" className="min-h-[280px]">
          <SelectionGroup<"Standby" | "ON">
            value={isRunning ? "ON" : "Standby"}
            disabled={state === null}
            className="grid flex-1 grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              ON: {
                children: "ON",
                icon: "lu:CirclePlay",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={(val) => setRunning(val === "ON")}
          />
          {statusInfo && (
            <div className="flex items-center gap-2 rounded-xl border border-gray-200 bg-gray-50 px-3 py-2 text-sm font-semibold text-gray-700">
              <Icon name={statusInfo.icon as any} className="size-4" />
              <span>{statusInfo.label}</span>
            </div>
          )}
          {alarmMessage && (
            <StatusBadge variant="error">{alarmMessage}</StatusBadge>
          )}
          {warningMessage && (
            <StatusBadge variant="error">{warningMessage}</StatusBadge>
          )}
        </ControlCard>

        <ControlCard title="Regeneration">
          <TimeSeriesValueNumeric
            label="Regen Inlet Temperature"
            unit="C"
            timeseries={ts_temp_regen_in}
            renderValue={(val) => val.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Regen Outlet Temperature"
            unit="C"
            timeseries={ts_temp_regen_out}
            renderValue={(val) => val.toFixed(1)}
          />
        </ControlCard>

        <ControlCard title="Air Flow">
          <TimeSeriesValueNumeric
            label="Fan Inlet Temperature"
            unit="C"
            timeseries={ts_temp_fan_inlet}
            renderValue={(val) => val.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Return Air Temperature"
            unit="C"
            timeseries={ts_temp_return_air}
            renderValue={(val) => val.toFixed(1)}
          />
          <div className="text-sm text-gray-400">
            Dew Point:{" "}
            <span className="font-mono font-semibold text-gray-700">
              {liveValues ? `${liveValues.temp_dew_point.toFixed(1)} C` : "—"}
            </span>
          </div>
        </ControlCard>

        <ControlCard title="Fans">
          <TimeSeriesValueNumeric
            label="Fan 1 PWM"
            unit="%"
            timeseries={ts_pwm_fan1}
            renderValue={(val) => val.toFixed(0)}
          />
          <TimeSeriesValueNumeric
            label="Fan 2 PWM"
            unit="%"
            timeseries={ts_pwm_fan2}
            renderValue={(val) => val.toFixed(0)}
          />
        </ControlCard>

        <ControlCard title="Power">
          <TimeSeriesValueNumeric
            label="Process Power"
            unit="W"
            timeseries={ts_power_process}
            renderValue={(val) => val.toFixed(0)}
          />
          <TimeSeriesValueNumeric
            label="Regen Power"
            unit="W"
            timeseries={ts_power_regen}
            renderValue={(val) => val.toFixed(0)}
          />
        </ControlCard>

        {state?.is_smart && (
          <ControlCard title="Hardware">
            <StatusBadge variant="success">Smart Dryer</StatusBadge>
          </ControlCard>
        )}
      </ControlGrid>
    </Page>
  );
}
