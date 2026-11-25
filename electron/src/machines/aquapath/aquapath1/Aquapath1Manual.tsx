import { Page } from "@/components/Page";
import { PdfManual } from "@/components/manual/PdfManual";
import React from "react";

export function Aquapath1ManualPage() {
  return (
    <Page>
      <PdfManual machine="aquapath" />
    </Page>
  );
}
