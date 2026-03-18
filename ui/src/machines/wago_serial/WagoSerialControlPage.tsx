import { Page } from "@ui/components/Page";
import React from "react";
import { useWagoSerial } from "./useWagoSerial";
import { ControlGrid } from "@ui/control/ControlGrid";
import { ControlCard } from "@ui/control/ControlCard";
import { TimeSeriesValueNumeric } from "@ui/control/TimeSeriesValue";
import { SelectionGroup } from "@ui/control/SelectionGroup";
import { useState } from "react";
import { Input } from "@ui/components/ui/input";
import { TouchButton } from "@ui/components/touch/TouchButton";

export function WagoSerialControlPage() {
  const [message, setMessage] = useState("");

  const { state, sendMessage, isLoading, isDisabled } = useWagoSerial();

  return (
    <Page>
      <Input
        placeholder="Message here, Maximum 22 characters"
        onChange={(e) => setMessage(e.target.value)}
        className="w-full"
      />

      <TouchButton
        variant="outline"
        icon="lu:X"
        className="h-21 flex-1"
        onClick={() => sendMessage(message)}
        disabled={isLoading || isDisabled}
      >
        Send Message
      </TouchButton>

      <div className="bg-muted flex min-h-[60px] items-center justify-center rounded-md border p-4">
        {state?.current_message ? (
          <span className="text-primary font-mono text-xl">
            {state.current_message}
          </span>
        ) : (
          <span className="text-muted-foreground italic">
            No data received yet
          </span>
        )}
      </div>
    </Page>
  );
}
