import { Topbar } from "@/components/Topbar";
import { ufmFlowTestMachineSerialRoute } from "@/routes/routes";
import React from "react";

export function UfmFlowTestMachinePage() {
  const { serial } = ufmFlowTestMachineSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/ufmflowtestmachine/${serial}`}
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
