import { Page } from "@/components/Page";
import { PdfManual } from "@/components/manual/PdfManual";
import React from "react";

export function ExtruderV2ManualPage() {
  return (
    <Page>
      <PdfManual machine="extruder" />
    </Page>
  );
}
