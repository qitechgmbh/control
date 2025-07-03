import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React from "react";

import {
  Mode,
} from "./buffer1Namespace";
import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";

export function Buffer1ControlPage() {
  const {
    mode,

    bufferSetMode,
    bufferGoUp,
    bufferGoDown,
    } = useBuffer1();

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

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "FillingBuffer" | "EmptyingBuffer">
            value={mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              FillingBuffer: {
                children: "FillingBuffer",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              EmptyingBuffer: {
                children: "EmptyingBuffer",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={bufferSetMode}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
