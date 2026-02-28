import { Topbar } from "@/components/Topbar";
import { wago750_501TestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750_501TestMachinePage() {
  const { serial } = wago750_501TestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750501testmachine/${serial}`}
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
