import { Page } from "@/components/Page";
import { winder2SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1GraphsPage() {
  const { serial } = winder2SerialRoute.useParams();
  return <Page>Winder 2 Graph</Page>;
}
