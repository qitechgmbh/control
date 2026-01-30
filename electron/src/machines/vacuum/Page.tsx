import { Topbar } from "@/components/Topbar";
import { vacuumSerialRoute } from "@/routes/routes";
import React from "react";

export function VacuumPage(): React.JSX.Element {
  const { serial } = vacuumSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/vacuum/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:ToggleLeft",
        },
      ]}
    />
  );
}
