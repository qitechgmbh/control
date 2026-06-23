import { Page } from "@/components/Page";
import { PageTabs } from "@/components/PageTabs";
import { wago671Slot12TestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago671Slot12TestMachinePage() {
  const { serial } = wago671Slot12TestMachineSerialRoute.useParams();

  return (
    <Page>
      <PageTabs
        serial={serial}
        machine_slug="wago671slot12testmachine"
        tabs={[
          {
            label: "Control",
            href: `/machines/wago671slot12testmachine/${serial}/control`,
          },
        ]}
      />
    </Page>
  );
}
