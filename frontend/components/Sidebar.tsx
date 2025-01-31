"use client";
import { Droplet, Flame, RotateCcw, Wrench } from "lucide-react";
import {} from "./ui/collapsible";
import { usePathname, useRouter } from "next/navigation";
import { OutsideCorner } from "./OutsideCorner";

type SidbarButton = {
  icon: React.ReactNode;
  children: React.ReactNode;
  // used for active state detection
  parentLink?: string;
  // actual navigation link
  link?: string;
  isFirst?: boolean;
};

function SidbarButton({
  icon,
  children,
  link,
  parentLink,
  isFirst,
}: SidbarButton) {
  const router = useRouter();
  // get current route
  const pathname = usePathname();
  const isActive = parentLink ? pathname.startsWith(parentLink) : false;

  return (
    <div
      className={`w-full h-20 relative ${isActive ? " pl-2" : " px-2"}`}
      onClick={() => (link && !isActive ? router.push(link) : null)}
    >
      <div
        className={`flex items-center justify-center w-full h-full gap-2 text-md z-10 relative ${
          isActive ? "bg-white rounded-l-lg" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon}
        {children}
      </div>
      <OutsideCorner rightTop={isActive && !isFirst} rightBottom={isActive} />
    </div>
  );
}

export function AppSidebar() {
  return (
    <>
      <div className="w-40"></div>
      <div className="flex flex-col h-full fixed w-40 z-10 bg-neutral-200">
        <div className="h-20">QiTech Control</div>
        <div className="flex flex-col gap-2">
          <SidbarButton icon={<RotateCcw size={22} />} isFirst>
            Winder
          </SidbarButton>
          <SidbarButton icon={<Flame size={22} />}>Extruder</SidbarButton>
          <SidbarButton icon={<Droplet size={22} />}>
            Wasserkühlung
          </SidbarButton>
          <SidbarButton
            icon={<Wrench size={22} />}
            parentLink="/configuration"
            link="/configuration/ethercat"
          >
            Konfiguration
          </SidbarButton>
        </div>
      </div>
    </>
  );
}
