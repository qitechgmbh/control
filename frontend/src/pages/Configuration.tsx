import { Topbar, TopbarItem } from "@/components/Topbar";
import { EthernetPort } from "lucide-react";
import EthercatPage from "./Ethercat";

export function ConfigurationPage() {
  return (
    <Topbar
      items={[
        {
          button: (props) => (
            <TopbarItem
              {...props}
              title="Ethercat"
              icon={<EthernetPort size={22} />}
            />
          ),
          children: <EthercatPage />,
        },
      ]}
    />
  );
}
