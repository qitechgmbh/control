import { Topbar } from "@/components/Topbar";
import { testMachineSerialRoute } from "@/routes/routes";
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
