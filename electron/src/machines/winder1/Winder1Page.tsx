import { Topbar } from "@/components/Topbar";
import { winder1SerialRoute } from "@/routes/routes";
import React from "react";

export function Winder1Page() {
  const { serial } = winder1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/winder1/${serial}`}
      items={[
        {
          link: "control",
          title: "Control",
        },
      ]}
    />
  );
}
