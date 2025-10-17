import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React, { useState } from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TraverseBar } from "../TraverseBar";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { useWinder2 } from "./useWinder";
import {
  Mode,
  SpoolAutomaticActionMode,
  getGearRatioMultiplier,
} from "./winder2Namespace";
import { TensionArm } from "../TensionArm";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { Spool } from "../Spool";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { getWinder2TraverseMax } from "./winder2Config";

export function Winder2ControlPage() {
  const [showResetConfirmDialog, setShowResetConfirmDialog] = useState(false);
  const traverseMax = getWinder2TraverseMax();

  // use optimistic state
  const {
    state,
    defaultState,
    enableTraverseLaserpointer,
    tensionArmAngle,
    zeroTensionArmAngle,
    spoolRpm,
    setMode,
    pullerSpeed,
    spoolProgress,
    setPullerRegulationMode,
    setPullerTargetSpeed,
    traversePosition,
    resetSpoolProgress,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseHome,
    setSpoolAutomaticRequiredMeters,
    setSpoolAutomaticAction,
    isLoading,
    isDisabled,
  } = useWinder2();

  // Calculate max speed based on gear ratio
  const gearRatioMultiplier = getGearRatioMultiplier(
    state?.puller_state?.gear_ratio,
  );
  const maxMotorSpeed = 75; // Maximum motor speed in m/min
  const maxTargetSpeed = maxMotorSpeed / gearRatioMultiplier;

  const handleResetProgress = () => {
    // Check if the machine is currently in Wind mode
    if (state?.mode_state?.mode === "Wind") {
      setShowResetConfirmDialog(true);
    } else {
      resetSpoolProgress();
    }
  };

  const confirmResetProgress = () => {
    resetSpoolProgress();
    setShowResetConfirmDialog(false);
  };

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Spool">
          <Spool rpm={spoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Spool Speed"
            unit="rpm"
            timeseries={spoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard className="bg-red" height={2} title="Traverse">
          <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            timeseries={traversePosition}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          {state?.traverse_state && (
            <TraverseBar
              inside={0}
              outside={traverseMax}
              min={state?.traverse_state.limit_inner}
              max={state?.traverse_state.limit_outer}
              current={traversePosition.current?.value ?? 0}
            />
          )}
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Outer Limit">
              <EditValue
                value={state?.traverse_state?.limit_outer}
                unit="mm"
                title="Outer Limit"
                defaultValue={defaultState?.traverse_state?.limit_outer}
                // Traverse limit validation: Outer limit must be at least 0.9mm greater than inner limit
                // We use 1mm buffer to ensure the backend validation (which requires >0.9mm) will pass
                // Formula: min_outer = inner_limit + 1mm
                min={Math.max(0, (state?.traverse_state?.limit_inner ?? 0) + 1)}
                minLabel="IN"
                maxLabel="OUT"
                max={traverseMax}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setTraverseLimitOuter}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowLeftToLine"
                onClick={gotoTraverseLimitOuter}
                disabled={isDisabled}
                isLoading={isLoading}
              >
                Go to Outer Limit
              </TouchButton>
            </Label>
            <Label label="Inner Limit">
              <EditValue
                value={state?.traverse_state?.limit_inner}
                unit="mm"
                title="Inner Limit"
                min={0}
                // Traverse limit validation: Inner limit must be at least 0.9mm smaller than outer limit
                // We use 1mm buffer to ensure the backend validation (which requires outer > inner + 0.9mm) will pass
                // Formula: max_inner = outer_limit - 1mm
                max={Math.min(
                  traverseMax,
                  (state?.traverse_state?.limit_outer ?? traverseMax) - 1,
                )}
                defaultValue={defaultState?.traverse_state?.limit_inner}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={setTraverseLimitInner}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowRightToLine"
                onClick={gotoTraverseLimitInner}
                disabled={isDisabled}
                isLoading={isLoading}
              >
                Go to Inner Limit
              </TouchButton>
            </Label>
          </div>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={state?.traverse_state.laserpointer}
              disabled={isLoading || isDisabled}
              loading={isLoading}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
              optionTrue={{ children: "On", icon: "lu:Lightbulb" }}
              onChange={enableTraverseLaserpointer}
            />
          </Label>
          <Label label="Home">
            <TouchButton
              variant="outline"
              icon="lu:House"
              onClick={() => gotoTraverseHome()}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Go to Home
            </TouchButton>
            {state?.traverse_state?.is_homed !== true ? (
              <StatusBadge variant={"error"}>{"Not Homed"}</StatusBadge>
            ) : null}
          </Label>
        </ControlCard>

        <ControlCard title="Tension Arm">
          <TensionArm degrees={tensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Tension Arm"
            unit="deg"
            timeseries={tensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroTensionArmAngle}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.tension_arm_state?.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Wind: {
                children: "Wind",
                icon: "lu:RefreshCcw",
                isActiveClassName: "bg-green-600",
                disabled: !state?.mode_state?.can_wind,
                className: "h-full",
              },
            }}
          />
        </ControlCard>

        <ControlCard className="bg-red" title="Puller">
          <TimeSeriesValueNumeric
            label="Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          <Label label="Regulation">
            <SelectionGroup
              value={state?.puller_state?.regulation}
              options={{
                Speed: {
                  children: "Speed",
                  icon: "lu:Gauge",
                },
                Diameter: {
                  children: "Diameter",
                  icon: "lu:Sun",
                  disabled: true,
                },
              }}
              onChange={setPullerRegulationMode}
              disabled={isDisabled}
              loading={isLoading}
            />
          </Label>
          <Label label="Target Speed">
            <EditValue
              value={state?.puller_state?.target_speed}
              unit="m/min"
              title="Target Speed"
              defaultValue={defaultState?.puller_state?.target_speed}
              min={0}
              max={maxTargetSpeed}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setPullerTargetSpeed}
            />
          </Label>
        </ControlCard>

        <ControlCard className="bg-red" title="Spool Autostop">
          <TimeSeriesValueNumeric
            label="Pulled Distance"
            renderValue={(value) => roundToDecimals(value, 2)}
            unit="m"
            timeseries={spoolProgress}
          />

          <Label label="Target Length">
            <EditValue
              value={state?.spool_automatic_action_state.spool_required_meters}
              unit="m"
              title="Expected Meters"
              defaultValue={250}
              min={10}
              max={10000}
              step={10}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setSpoolAutomaticRequiredMeters}
            />
          </Label>

          <TouchButton
            variant="outline"
            onClick={handleResetProgress}
            disabled={isDisabled}
            isLoading={isLoading || state?.traverse_state?.is_going_out}
          >
            Reset Progress
          </TouchButton>

          <Label label="After Target Length Reached">
            <SelectionGroup<SpoolAutomaticActionMode>
              value={
                state?.spool_automatic_action_state.spool_automatic_action_mode
              }
              disabled={isDisabled}
              loading={isLoading}
              onChange={setSpoolAutomaticAction}
              orientation="vertical"
              options={{
                Hold: {
                  children: "Hold",
                  icon: "lu:CirclePause",
                  className: "h-full",
                },
                Pull: {
                  children: "Pull",
                  icon: "lu:ChevronsLeft",
                  className: "h-full",
                },

                NoAction: {
                  children: "No Action",
                  icon: "lu:RefreshCcw",
                  className: "h-full",
                },
              }}
            />
          </Label>
        </ControlCard>
      </ControlGrid>

      {/* Reset Progress Confirmation Dialog */}
      <Dialog
        open={showResetConfirmDialog}
        onOpenChange={setShowResetConfirmDialog}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Reset Spool Progress?</DialogTitle>
            <DialogDescription>
              The machine is currently in Wind mode. Are you sure you want to
              reset the spool progress?
            </DialogDescription>
          </DialogHeader>

          <div className="mt-4 flex flex-col gap-2">
            <TouchButton
              variant="destructive"
              onClick={confirmResetProgress}
              disabled={isLoading}
              isLoading={isLoading}
            >
              Yes, Reset Progress
            </TouchButton>
            <TouchButton
              variant="outline"
              onClick={() => setShowResetConfirmDialog(false)}
            >
              Cancel
            </TouchButton>
          </div>
        </DialogContent>
      </Dialog>
    </Page>
  );
}
