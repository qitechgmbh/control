import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { StatusBadge } from "@/control/StatusBadge";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";
import React from "react";
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

function Measurement({
  label,
  value,
  unit,
  decimals = 1,
}: {
  label: string;
  value: number | undefined;
  unit: string;
  decimals?: number;
}) {
  return (
    <div className="flex items-center justify-between text-sm">
      <span className="text-gray-500">{label}</span>
      <strong className="font-mono">
        {value !== undefined ? `${value.toFixed(decimals)} ${unit}` : "—"}
      </strong>
    </div>
  );
}

export function DryerV1ControlPage() {
  const {
    state,
    liveValues,
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

  return (
    <Page>
      <ControlGrid columns={3}>
        <ControlCard title="Mode" className="min-h-[280px]">
          <TouchButton
            icon={isRunning ? "lu:Pause" : "lu:Play"}
            disabled={state === null}
            className={isRunning ? "" : "bg-green-600 text-white"}
            variant={isRunning ? "destructive" : undefined}
            onClick={() => setRunning(!isRunning)}
          >
            {isRunning ? "Stop" : "Start"}
          </TouchButton>
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

        <ControlCard title="Timer">
          <div className="flex items-center gap-1.5 text-sm text-gray-400">
            <Icon name="lu:Clock" className="size-4" />
            <span>Remaining Time</span>
          </div>
          <div className="text-3xl font-bold">
            {remainingSec !== null ? formatRemaining(remainingSec) : "—"}
          </div>
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
          <p className="text-xs text-gray-400">
            Only used on days with no scheduled stop time.
          </p>
        </ControlCard>

        <ControlCard title="Temperature">
          <Measurement
            label="Process Temperature"
            value={liveValues?.temp_process}
            unit="C"
          />
          <Measurement
            label="Safety Temperature"
            value={liveValues?.temp_safety}
            unit="C"
          />
          <Label label="Target Temperature">
            <EditValue
              title="Target Temperature"
              value={state?.target_temperature}
              min={50}
              max={180}
              step={1}
              unit="C"
              disabled={state === null}
              renderValue={(val) => val.toFixed(0)}
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

        <ControlCard title="Regeneration">
          <Measurement
            label="Regen Inlet Temperature"
            value={liveValues?.temp_regen_in}
            unit="C"
          />
          <Measurement
            label="Regen Outlet Temperature"
            value={liveValues?.temp_regen_out}
            unit="C"
          />
        </ControlCard>

        <ControlCard title="Air Flow">
          <Measurement
            label="Fan Inlet Temperature"
            value={liveValues?.temp_fan_inlet}
            unit="C"
          />
          <Measurement
            label="Return Air Temperature"
            value={liveValues?.temp_return_air}
            unit="C"
          />
          <Measurement
            label="Dew Point"
            value={liveValues?.temp_dew_point}
            unit="C"
          />
        </ControlCard>

        <ControlCard title="Fans">
          <Measurement
            label="Fan 1 PWM"
            value={liveValues?.pwm_fan1}
            unit="%"
            decimals={0}
          />
          <Measurement
            label="Fan 2 PWM"
            value={liveValues?.pwm_fan2}
            unit="%"
            decimals={0}
          />
        </ControlCard>

        <ControlCard title="Power">
          <Measurement
            label="Process Power"
            value={liveValues?.power_process}
            unit="W"
            decimals={0}
          />
          <Measurement
            label="Regen Power"
            value={liveValues?.power_regen}
            unit="W"
            decimals={0}
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
