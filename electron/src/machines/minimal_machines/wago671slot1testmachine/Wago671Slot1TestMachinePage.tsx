import { Topbar } from "@/components/Topbar";
import { wago671Slot1TestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago671Slot1TestMachinePage() {
  const { serial } = wago671Slot1TestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago671slot1testmachine/${serial}`}
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
