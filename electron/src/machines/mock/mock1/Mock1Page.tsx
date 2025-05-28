import { Topbar } from "@/components/Topbar";
import { mock1SerialRoute } from "@/routes/routes";
import React from "react";

export function Mock1Page() {
  const { serial } = mock1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/mock1/${serial}`}
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
