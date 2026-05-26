import { Topbar } from "@/components/Topbar";
import { analogOutOversamplingSerialRoute } from "@/routes/routes";
import React from "react";

export function AnalogOutOversamplingPage(): React.JSX.Element {
  const { serial } = analogOutOversamplingSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/analogoutoversampling/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:AudioWaveform",
        },
      ]}
    />
  );
}
