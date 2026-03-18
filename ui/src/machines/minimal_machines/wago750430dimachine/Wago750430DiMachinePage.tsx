import { Topbar } from "@ui/components/Topbar";
import { wago750430DiMachineSerialRoute } from "@ui/routes/routes";
import React from "react";

export function Wago750430DiMachinePage() {
  const { serial } = wago750430DiMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750430dimachine/${serial}`}
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
