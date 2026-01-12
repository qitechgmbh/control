import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { useTestMachineStepper } from "./useTestMachineStepper";

export function TestMachineStepperControlPage() {
  const { state } = useTestMachineStepper();


  return (
    <Page>
      <ControlGrid columns={2}>
      </ControlGrid>
    </Page>
  );
}
