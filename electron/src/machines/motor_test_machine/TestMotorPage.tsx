import { Topbar } from "@/components/Topbar";
import { testMotorSerialRoute } from "@/routes/routes";
import React from "react";

export function TestMotorPage() {
  // Hier ist der Zugriff erlaubt, weil diese Funktion erst später aufgerufen wird
  const { serial } = testMotorSerialRoute.useParams();

  return (
    <Topbar
      pathname={`/_sidebar/machines/testmotor/${serial}`}
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
