import { Page } from "@/components/Page";
import { PdfManual } from "@/components/manual/PdfManual";
import React from "react";

export function ExtruderV3ManualPage() {
  return (
    <Page>
      <PdfManual machine="extruder" />
    </Page>
  );
}
