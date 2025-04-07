import { Page } from "@/components/Page";
import { winder2SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1SettingPage() {
  const { serial } = winder2SerialRoute.useParams();
  return <Page>Winder 2 Settings</Page>;
}
