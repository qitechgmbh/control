import { Topbar } from "@/components/Topbar";
import { dre1SerialRoute } from "@/routes/routes";
import React from "react";

export function Dre1Page() {
    const { serial } = dre1SerialRoute.useParams();
    return (
        <Topbar
            pathname={`/_sidebar/machines/dre1/${serial}`}
            items={[
                {
                    link: "control",
                    activeLink: "control",
                    title: "Steuerung",
                    icon: "lu:CirclePlay",
                },
                {
                    link: "graphs",
                    activeLink: "graphs",
                    title: "Graphs",
                    icon: "lu:ChartSpline",
                },
            ]}
        />
    );
}
