import { Topbar } from "@/components/Topbar";
import { wago750467MachineSerialRoute } from "@/routes/routes";
import React from "react";

export function Wago750467MachinePage(): React.JSX.Element {
  const { serial } = wago750467MachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/wago750467machine/${serial}`}
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
