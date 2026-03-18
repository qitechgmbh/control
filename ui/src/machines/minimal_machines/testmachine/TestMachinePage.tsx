import { Topbar } from "@ui/components/Topbar";
import { testMachineSerialRoute } from "@ui/routes/routes";
import React from "react";

export function TestMachinePage() {
  const { serial } = testMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/testmachine/${serial}`}
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
