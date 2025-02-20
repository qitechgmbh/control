import { OutsideCorner } from "@/components/OutsideCorner";
import { useOnSubpath } from "@/hooks/useOnSubpath";
import { Link, Outlet } from "@tanstack/react-router";
import { EthernetPort } from "lucide-react";
import { Fragment } from "react";
import React from "react";

type SidebarItemContent = {
  link: string;
  icon?: React.ReactNode;
  title: string;
};

type SidebarItemProps = SidebarItemContent & {
  isFirst: boolean;
};

export function SidebarItem({ link, icon, title, isFirst }: SidebarItemProps) {
  const isActive = useOnSubpath(link);
  return (
    <Link
      to={link}
      className={`relative h-20 w-full ${isActive ? "pl-2" : "px-2"}`}
    >
      <div
        className={`text-md relative z-10 flex h-full w-full items-center justify-center gap-2 ${
          isActive ? "rounded-l-lg bg-white" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon}
        {title}
      </div>
      {isActive && <OutsideCorner rightTop={!isFirst} rightBottom={true} />}
    </Link>
  );
}

export function SidebarLayout() {
  const items: SidebarItemContent[] = [
    {
      link: "/_sidebar/machines/123",
      title: "Winder 123",
    },
    {
      link: "/_sidebar/configuration/devices",
      title: "Ethercat",
      icon: <EthernetPort size={20} />,
    },
  ];

  return (
    <div className="flex h-full flex-row">
      <div className="w-40" />
      <div className="fixed flex h-full w-40 flex-col bg-neutral-200">
        <div className="flex h-20 flex-col items-center justify-center gap-0 pt-2">
          <div className="font-qitech line-clamp-none text-3xl">QiTech</div>
        </div>
        <div className="flex flex-col gap-2">
          {items.map((item, index) => (
            <Fragment key={item.link}>
              <SidebarItem {...item} isFirst={index === 0} />
            </Fragment>
          ))}
        </div>
      </div>
      <Outlet />
    </div>
  );
}
