import { Page } from "@/components/Page";
import { extruder2Route } from "@/routes/routes";
import React from "react";

export function ExtruderV2ManualPage() {
  const { serial } = extruder2Route.useParams();
  return <Page>Extruder V2 Manual</Page>;
}
