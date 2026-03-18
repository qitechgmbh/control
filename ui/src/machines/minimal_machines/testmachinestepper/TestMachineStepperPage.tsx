import { Topbar } from "@ui/components/Topbar";
import { testMachineStepperSerialRoute } from "@ui/routes/routes";
import React from "react";

export function TestMachineStepperPage() {
  const { serial } = testMachineStepperSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/testmachinestepper/${serial}`}
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
