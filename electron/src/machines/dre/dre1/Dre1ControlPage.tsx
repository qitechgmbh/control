
import React from "react";
import { Topbar } from "@/components/Topbar";

export function Dre1ControlPage() {
    return (
        <Topbar
            pathname="/_sidebar/dre"
            items={[
                {
                    link: "machines",
                    activeLink: "machines",
                    title: "Machines",
                    icon: "lu:Factory",
                },
                {
                    link: "ethercat",
                    activeLink: "ethercat",
                    title: "EtherCat",
                    icon: "lu:EthernetPort",
                },

                {
                    link: "update/choose-version",
                    activeLink: "update",
                    title: "Update",
                    icon: "lu:CircleFadingArrowUp",
                },
            ]}
        />
    );
}
