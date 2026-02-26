import { Topbar } from "@/components/Topbar";
import { testMachineStepperSerialRoute } from "@/routes/routes";
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
