import { Page } from "@/components/Page";
import { PdfManual } from "@/components/manual/PdfManual";
import React from "react";

export function Winder2ManualPage() {
  return (
    <Page>
      <PdfManual machine="winder" />
    </Page>
  );
}
