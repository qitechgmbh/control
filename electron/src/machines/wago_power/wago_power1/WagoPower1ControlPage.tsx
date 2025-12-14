import { Page } from "@/components/Page";
import React from "react";
import { useWagoPower1Namespace } from "./wagoPower1Namespace";
import { useWagoPower1 } from "./useWagoPower1";

export function WagoPower1ControlPage() {
  const { state } = useWagoPower1();
  return (
    <Page>
      <h1>{ state?.data?.voltage_milli_volt ?? "N/A" } mV</h1>
      <h1>{ state?.data?.current_milli_ampere ?? "N/A" } mA</h1>
    </Page>
  );
}
