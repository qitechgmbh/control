import { Topbar } from "@/components/Topbar";
import { EthernetPort } from "lucide-react";

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  return (
    <div>
      <Topbar
        items={[
          {
            icon: <EthernetPort size={22} />,
            title: "Ethercat",
            link: "/configuration/ethercat",
          },
          { title: "Other", link: "/configuration/other" },
        ]}
      />
      <div className="overflow-hidden">{children}</div>
    </div>
  );
}
