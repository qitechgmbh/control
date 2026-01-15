import { Topbar } from "@/components/Topbar";
import { digitalInputTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function DigitalInputTestMachinePage() {
  const { serial } = digitalInputTestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/digitalinputtestmachine/${serial}`}
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
