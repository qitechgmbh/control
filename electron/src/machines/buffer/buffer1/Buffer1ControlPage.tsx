import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React from "react";

import { useBuffer1 } from "./useBuffer1";

export function Buffer1ControlPage() {
  const { bufferGoUp, bufferGoDown } = useBuffer1();
  return (
    <Page>
      <ControlGrid>
        <Label label="Buffer go up!">
          <TouchButton
            variant="outline"
            icon="lu:ArrowUpToLine"
            onClick={() => bufferGoUp()}
          ></TouchButton>
        </Label>
        <Label label="Buffer go down!">
          <TouchButton
            variant="outline"
            icon="lu:ArrowDownToLine"
            onClick={() => bufferGoDown()}
          ></TouchButton>
        </Label>
      </ControlGrid>
    </Page>
  );
}
