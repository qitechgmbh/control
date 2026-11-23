import { Topbar } from "@/components/layout/Topbar";
import { wagoSerialSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoSerialPage() {
  const { serial } = wagoSerialSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wago_serial/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:PlugZap",
        },
      ]}
    />
  );
}
