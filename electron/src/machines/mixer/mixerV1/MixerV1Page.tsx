import { Topbar } from "@/components/Topbar";
import { mixerV1SerialRoute } from "@/routes/routes";
import React from "react";

export function MixerV1Page() {
  const { serial } = mixerV1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/mixerV1/${serial}`}
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
