// ============================================================================
// MyMachineControlPage.tsx — Control UI for this machine
// ============================================================================
// This component is the main control panel rendered under the "Control" tab.
// It reads machine state from the hook and dispatches mutations on user action.
//
// Layout components:
//   Page         — full-height scrollable container
//   ControlGrid  — responsive column grid (columns prop)
//   ControlCard  — card with a title header
//   Label        — labelled wrapper for a single control
//
// Common control components (import as needed):
//   SelectionGroup — radio-style button group
//   Slider         — numeric range input
//   Toggle         — on/off switch
//   NumericInput   — free-form number input
//
// FIND & REPLACE to adapt this template:
//   MyMachine    → YourMachineName
//   myMachine    → yourMachineName
//   useMyMachine → useYourMachineName
// ============================================================================

import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";
import { useMyMachine } from "./useMyMachine";

export function MyMachineControlPage() {
  // Pull state and mutation functions from the hook.
  // Add your mutation functions to the destructuring as you implement them.
  const { state } = useMyMachine();

  // Provide a safe default so the UI renders before the first server event.
  // TODO: replace with the actual shape of your StateEvent.
  const safeState =
    state ??
    {
      // e.g. outputs: [false, false, false, false],
      //      value:   0,
    };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* ------------------------------------------------------------------ */}
        {/* TODO: replace the example card below with your real controls.       */}
        {/*                                                                      */}
        {/* Example: four digital outputs                                        */}
        {/* ------------------------------------------------------------------ */}
        <ControlCard title="Outputs">
          <p className="text-muted-foreground text-sm">
            {/* Replace this placeholder with real controls, e.g.: */}
          </p>

          {/* Example — toggle individual outputs:
          <div className="grid grid-cols-2 gap-6">
            {safeState.outputs.map((on, index) => (
              <Label key={index} label={`Output ${index + 1}`}>
                <SelectionGroup<"On" | "Off">
                  value={on ? "On" : "Off"}
                  orientation="vertical"
                  className="flex flex-col gap-3"
                  options={{
                    Off: {
                      children: "Off",
                      icon: "lu:CirclePause",
                      isActiveClassName: "bg-red-600",
                      className: "flex-1",
                    },
                    On: {
                      children: "On",
                      icon: "lu:CirclePlay",
                      isActiveClassName: "bg-green-600",
                      className: "flex-1",
                    },
                  }}
                  onChange={(value) => setOutput(index, value === "On")}
                />
              </Label>
            ))}
          </div>
          */}
        </ControlCard>

        {/* ------------------------------------------------------------------ */}
        {/* Example: master control card                                         */}
        {/* ------------------------------------------------------------------ */}
        {/* <ControlCard title="Master Control">
          <SelectionGroup<"On" | "Off">
            value={safeState.outputs.every(Boolean) ? "On" : "Off"}
            orientation="horizontal"
            options={{
              Off: { children: "Turn All Off" },
              On: { children: "Turn All On" },
            }}
            onChange={(value) => setAllOutputs(value === "On")}
          />
        </ControlCard> */}

        {/* TODO: add more ControlCard sections for your machine */}
      </ControlGrid>
    </Page>
  );
}
