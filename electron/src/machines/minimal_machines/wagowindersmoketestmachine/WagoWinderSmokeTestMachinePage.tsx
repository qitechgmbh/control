import { Topbar } from "@/components/Topbar";
import { wagoWinderSmokeTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoWinderSmokeTestMachinePage() {
  const { serial } = wagoWinderSmokeTestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/steppercontrollertester/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
      ]}
    />
  );
}
