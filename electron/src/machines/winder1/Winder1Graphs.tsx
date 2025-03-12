import { Page } from "@/components/Page";
import { winder1SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1GraphsPage() {
  const { serial } = winder1SerialRoute.useParams();
  return <Page>Winder 1 Graph</Page>;
}
