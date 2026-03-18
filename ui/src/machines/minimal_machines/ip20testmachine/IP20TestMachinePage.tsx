import { Topbar } from "@ui/components/Topbar";
import { ip20TestMachineSerialRoute } from "@ui/routes/routes";
import React from "react";

export function IP20TestMachinePage(): React.JSX.Element {
  const { serial } = ip20TestMachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/ip20testmachine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:ToggleLeft",
        },
      ]}
    />
  );
}
