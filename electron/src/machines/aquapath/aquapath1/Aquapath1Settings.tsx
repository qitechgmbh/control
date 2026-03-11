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
    setFrontRevolutions,
    setBackRevolutions,
    setFrontHeatingTolerance,
    setFrontCoolingTolerance,
    setBackHeatingTolerance,
    setBackCoolingTolerance,
    setAmbientTemperatureCalibration,
    setFrontPidKp,
    setFrontPidKi,
    setFrontPidKd,
    setBackPidKp,
    setBackPidKi,
    setBackPidKd,
  } = useAquapath1();
  const frontTemp = state?.temperature_states.front.temperature;
  const backTemp = state?.temperature_states.back.temperature;
  const currentSensorAmbientCandidate =
    frontTemp != null && backTemp != null
      ? Math.min(frontTemp, backTemp)
      : undefined;
  const canApplySensorAmbient =
    currentSensorAmbientCandidate != null && currentSensorAmbientCandidate < 30;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Reservoir 1 (Back) Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
              min={0}
              value={state?.fan_states.back.max_revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Reservoir 2 (Front) Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Max Revolution Speed"
              min={0}
              value={state?.fan_states.front.max_revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontRevolutions(val);
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

        <ControlCard title="Reservoir 1 (Back) Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.back.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.back.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackCoolingTolerance(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setBackHeatingTolerance(DEFAULT_HEATING_TOLERANCE_C);
                setBackCoolingTolerance(DEFAULT_COOLING_TOLERANCE_C);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Reservoir 2 (Front) Temperature Tolerances">
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.front.heating}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Cooling Tolerance">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.front.cooling}
              max={10}
              step={0.1}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontCoolingTolerance(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setFrontHeatingTolerance(DEFAULT_HEATING_TOLERANCE_C);
                setFrontCoolingTolerance(DEFAULT_COOLING_TOLERANCE_C);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Reservoir 1 (Back) PID Settings">
          <Label label="Set Kp">
            <EditValue
              title="Set Reservoir 1 Kp"
              min={0}
              value={state?.pid_states.back.kp}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setBackPidKp(val);
              }}
            />
          </Label>

          <Label label="Set Ki">
            <EditValue
              title="Set Reservoir 1 Ki"
              min={0}
              value={state?.pid_states.back.ki}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setBackPidKi(val);
              }}
            />
          </Label>

          <Label label="Set Kd">
            <EditValue
              title="Set Reservoir 1 Kd"
              min={0}
              value={state?.pid_states.back.kd}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setBackPidKd(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setBackPidKp(DEFAULT_PID_KP);
                setBackPidKi(DEFAULT_PID_KI);
                setBackPidKd(DEFAULT_PID_KD);
              }}
            >
              Reset to Default
            </Button>
          </div>
        </ControlCard>

        <ControlCard title="Reservoir 2 (Front) PID Settings">
          <Label label="Set Kp">
            <EditValue
              title="Set Reservoir 2 Kp"
              min={0}
              value={state?.pid_states.front.kp}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setFrontPidKp(val);
              }}
            />
          </Label>

          <Label label="Set Ki">
            <EditValue
              title="Set Reservoir 2 Ki"
              min={0}
              value={state?.pid_states.front.ki}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setFrontPidKi(val);
              }}
            />
          </Label>

          <Label label="Set Kd">
            <EditValue
              title="Set Reservoir 2 Kd"
              min={0}
              value={state?.pid_states.front.kd}
              max={5}
              step={0.01}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                setFrontPidKd(val);
              }}
            />
          </Label>
          <div className="mt-3">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                setFrontPidKp(DEFAULT_PID_KP);
                setFrontPidKi(DEFAULT_PID_KI);
                setFrontPidKd(DEFAULT_PID_KD);
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
