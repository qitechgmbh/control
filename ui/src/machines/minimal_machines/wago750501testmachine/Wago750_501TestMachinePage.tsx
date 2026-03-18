import { Topbar } from "@ui/components/Topbar";
import { wago750_501TestMachineSerialRoute } from "@ui/routes/routes";
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
