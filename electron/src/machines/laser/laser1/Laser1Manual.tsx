import { Page } from "@/components/Page";
import { PdfManual } from "@/components/manual/PdfManual";
import React from "react";

export function Laser1ManualPage() {
  return (
    <Page>
      <PdfManual machine="laser" />
    </Page>
  );
}
