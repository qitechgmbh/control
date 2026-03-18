import { Topbar } from "@ui/components/Topbar";
import { analogInputTestMachineSerialRoute } from "@ui/routes/routes";
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
