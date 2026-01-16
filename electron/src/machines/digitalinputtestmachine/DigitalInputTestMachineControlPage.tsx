import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useDigitalInputTestMachine } from "./useDigitalInputTestMachine";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function DigitalInputTestMachineControlPage() {
  const { state } = useDigitalInputTestMachine();

  const safeState = state ?? { led_on: [false, false, false, false] };

  return (
  <Page>
    <LoadingSpinner></LoadingSpinner>
    </Page>
  );
}
