import { Page } from "@/components/Page";
import React from "react";
import { useWagoSerial } from "./useWagoSerial";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { SelectionGroup } from "@/control/SelectionGroup";
import { useState } from "react";
import { Input } from "@/components/ui/input";
import { TouchButton } from "@/components/touch/TouchButton";


export function WagoSerialControlPage() {
  const [message, setMessage] = useState("");
  
  const {
    state,
    sendMessage,
    isLoading,
    isDisabled,
  } = useWagoSerial();

  return (
    <Page>
     <Input
          placeholder="Message here, Maximum 24 characters"
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


    </Page>
  );
}
