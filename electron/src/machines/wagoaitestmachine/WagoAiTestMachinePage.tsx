import { Topbar } from "@/components/Topbar";
import { wagoAiTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoAiTestMachine(): React.JSX.Element {
  const { serial } = wagoAiTestMachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/wagoaitestmachine/${serial}`}
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
