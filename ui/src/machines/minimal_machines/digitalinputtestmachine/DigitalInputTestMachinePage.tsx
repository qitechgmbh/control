import { Topbar } from "@ui/components/Topbar";
import { digitalInputTestMachineSerialRoute } from "@ui/routes/routes";
import React from "react";

export function DigitalInputTestMachinePage() {
  const { serial } = digitalInputTestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/digitalInputTestMachine/${serial}`}
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
