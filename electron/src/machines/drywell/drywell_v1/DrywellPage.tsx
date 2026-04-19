import { Topbar } from "@/components/Topbar";
import { drywellV1SerialRoute } from "@/routes/routes";
import React from "react";

export function DrywellPage() {
  const { serial } = drywellV1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/drywell_v1/${serial}`}
      items={[
        {
          link: "control",
          title: "Control",
          icon: "lu:Thermometer",
          activeLink: "control",
        },
        {
          link: "overview",
          title: "Overview",
          icon: "lu:ScanEye",
          activeLink: "overview",
        },
      ]}
    />
  );
}
