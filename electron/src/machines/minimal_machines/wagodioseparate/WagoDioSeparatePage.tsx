import { Topbar } from "@/components/Topbar";
import { wagoDioSeparateSerialRoute } from "@/routes/routes";
import React from "react";

export function WagoDioSeparatePage() {
  const { serial } = wagoDioSeparateSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/wagodioseparate/${serial}`}
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