import { Topbar } from "@/components/Topbar";
import { wagoTraverseTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoTraverseTestMachinePage() {
  const { serial } = wagoTraverseTestMachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wagotraversetestmachine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:MoveHorizontal",
        },
      ]}
    />
  );
}
