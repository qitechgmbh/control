import { Topbar } from "@/components/Topbar";
import { analogInputTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function AnalogInputTestMachine(): React.JSX.Element {
  const { serial } = analogInputTestMachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/analoginputtestmachine/${serial}`}
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
