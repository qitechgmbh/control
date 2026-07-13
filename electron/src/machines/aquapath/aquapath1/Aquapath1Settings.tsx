import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { useAquapath1 } from "./useAquapath";
import React from "react";
import { Button } from "@/components/ui/button";

export function Aquapath1SettingsPage() {
  const DEFAULT_HEATING_TOLERANCE_C = 0.4;
  const DEFAULT_COOLING_TOLERANCE_C = 0.8;
  const DEFAULT_PID_KP = 0.16;
  const DEFAULT_PID_KI = 0.02;
  const DEFAULT_PID_KD = 0.0;

  const {
    state,
    setLeftRevolutions,
    setRightRevolutions,
    setLeftHeatingTolerance,
    setLeftCoolingTolerance,
    setRightHeatingTolerance,
    setRightCoolingTolerance,
    setAmbientTemperatureCalibration,
    setLeftPidKp,
    setLeftPidKi,
    setLeftPidKd,
    setRightPidKp,
    setRightPidKi,
    setRightPidKd,
    setLeftThermalFlowSettleDuration,
    setRightThermalFlowSettleDuration,
    setLeftPumpCooldownMinTemperature,
    setRightPumpCooldownMinTemperature,
  } = useAquapath1();
  const leftTemp = state?.temperature_states.left.temperature;
  const rightTemp = state?.temperature_states.right.temperature;
  const isStandby = state?.mode_state.mode === "Standby";
  const currentSensorAmbientCandidate =
    leftTemp != null && rightTemp != null
      ? Math.min(leftTemp, rightTemp)
      : undefined;
  const canApplySensorAmbient =
    currentSensorAmbientCandidate != null && currentSensorAmbientCandidate < 30;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Left Reservoir Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
              min={0}
              value={state?.fan_states.left.max_revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setLeftRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Right Reservoir Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
              min={0}
              value={state?.fan_states.right.max_revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setRightRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Ambient Calibration">
          <Label label="Set Ambient Temperature">
            <EditValue
              title="Set Ambient Temperature"
              min={10}
              value={state?.ambient_temperature_calibration ?? 22}
              max={40}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setAmbientTemperatureCalibration(val);
              }}
            />
          </Label>
          <div className="mt-3 flex items-center gap-3">
            <Button
              type="button"
              size="sm"
              disabled={!canApplySensorAmbient}
              onClick={() => {
                if (currentSensorAmbientCandidate == null) return;
                setAmbientTemperatureCalibration(currentSensorAmbientCandidate);
              }}
            >
              Use Current Sensor Temp
            </Button>
            <span className="text-muted-foreground text-sm">
              Candidate:{" "}
              {currentSensorAmbientCandidate != null
                ? `${currentSensorAmbientCandidate.toFixed(1)} C`
                : "N/A"}
            </span>
          </div>
        </ControlCard>

        <ControlCard title="Left Reservoir Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.left.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setLeftHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.left.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setLeftCoolingTolerance(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setLeftHeatingTolerance(DEFAULT_HEATING_TOLERANCE_C);
                setLeftCoolingTolerance(DEFAULT_COOLING_TOLERANCE_C);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Left Reservoir Thermal Flow Safety">
          <Label label="Thermal Safety Delay">
            <EditValue
              title="Set Left Reservoir Thermal Safety Delay"
              min={0}
              value={state?.thermal_safety_states.left.thermal_delay}
              max={30}
              step={0.5}
              unit="s"
              renderValue={(value) => value.toFixed(1)}
              disabled={!isStandby}
              onChange={(val) => {
                setLeftThermalFlowSettleDuration(val);
              }}
            />
          </Label>

          {!isStandby && (
            <p className="text-muted-foreground mt-3 text-sm">
              Thermal safety settings cannot be modified unless the controller
              is in standby.
            </p>
          )}

          <Label label="Pump Cooldown Min Temperature">
            <EditValue
              title="Set Left Reservoir Pump Cooldown Min Temperature"
              min={10}
              value={state?.thermal_safety_states.left.cooldown_min_temperature}
              max={80}
              step={0.5}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              disabled={!isStandby}
              onChange={(val) => {
                setLeftPumpCooldownMinTemperature(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Right Reservoir Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.right.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setRightHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.right.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setRightCoolingTolerance(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setRightHeatingTolerance(DEFAULT_HEATING_TOLERANCE_C);
                setRightCoolingTolerance(DEFAULT_COOLING_TOLERANCE_C);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Right Reservoir Thermal Flow Safety">
          <Label label="Thermal Safety Delay">
            <EditValue
              title="Set Right Reservoir Thermal Safety Delay"
              min={0}
              value={state?.thermal_safety_states.right.thermal_delay}
              max={30}
              step={0.5}
              unit="s"
              renderValue={(value) => value.toFixed(1)}
              disabled={!isStandby}
              onChange={(val) => {
                setRightThermalFlowSettleDuration(val);
              }}
            />
          </Label>

          {!isStandby && (
            <p className="text-muted-foreground mt-3 text-sm">
              Thermal safety settings cannot be modified unless the controller
              is in standby.
            </p>
          )}

          <Label label="Pump Cooldown Min Temperature">
            <EditValue
              title="Set Right Reservoir Pump Cooldown Min Temperature"
              min={10}
              value={
                state?.thermal_safety_states.right.cooldown_min_temperature
              }
              max={80}
              step={0.5}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              disabled={!isStandby}
              onChange={(val) => {
                setRightPumpCooldownMinTemperature(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Left Reservoir PID Settings">
          <Label label="Set Kp">
            <EditValue
              title="Set Left Reservoir Kp"
              min={0}
              value={state?.pid_states.left.kp}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setLeftPidKp(val);
              }}
            />
          </Label>

          <Label label="Set Ki">
            <EditValue
              title="Set Left Reservoir Ki"
              min={0}
              value={state?.pid_states.left.ki}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setLeftPidKi(val);
              }}
            />
          </Label>

          <Label label="Set Kd">
            <EditValue
              title="Set Left Reservoir Kd"
              min={0}
              value={state?.pid_states.left.kd}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setLeftPidKd(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setLeftPidKp(DEFAULT_PID_KP);
                setLeftPidKi(DEFAULT_PID_KI);
                setLeftPidKd(DEFAULT_PID_KD);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Right Reservoir PID Settings">
          <Label label="Set Kp">
            <EditValue
              title="Set Right Reservoir Kp"
              min={0}
              value={state?.pid_states.right.kp}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setRightPidKp(val);
              }}
            />
          </Label>

          <Label label="Set Ki">
            <EditValue
              title="Set Right Reservoir Ki"
              min={0}
              value={state?.pid_states.right.ki}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setRightPidKi(val);
              }}
            />
          </Label>

          <Label label="Set Kd">
            <EditValue
              title="Set Right Reservoir Kd"
              min={0}
              value={state?.pid_states.right.kd}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setRightPidKd(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setRightPidKp(DEFAULT_PID_KP);
                setRightPidKi(DEFAULT_PID_KI);
                setRightPidKd(DEFAULT_PID_KD);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
