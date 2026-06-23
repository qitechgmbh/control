import { Topbar } from "@/components/Topbar";
import { wago671Slot12TestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago671Slot12TestMachinePage() {
  const { serial } = wago671Slot12TestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago671slot12testmachine/${serial}`}
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
