import { Page } from "@/components/Page";
import { winder1SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1ManualPage() {
  const { serial } = winder1SerialRoute.useParams();
  return <Page>Winder 1 Manual</Page>;
}
