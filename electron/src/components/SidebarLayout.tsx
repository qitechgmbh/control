import { OutsideCorner } from "@/components/OutsideCorner";
import { useMachines } from "@/hooks/useMachines";
import { useOnSubpath } from "@/hooks/useOnSubpath";
import { Link, Outlet } from "@tanstack/react-router";
import { Fragment } from "react";
import React from "react";
import { Icon, IconName } from "./Icon";

type SidebarItemContent = {
  link: string;
  activeLink: string;
  icon?: IconName;
  title: string;
};

type SidebarItemProps = SidebarItemContent & {
  isFirst: boolean;
};

export function SidebarItem({
  link,
  icon,
  title,
  isFirst,
  activeLink,
}: SidebarItemProps) {
  const isActive = useOnSubpath(activeLink);
  return (
    <Link
      to={link}
      className={`relative h-20 w-full ${isActive ? "pl-2" : "px-2"}`}
    >
      <div
        className={`text-md relative z-10 flex h-full w-full items-center justify-center gap-2 ${
          isActive ? "rounded-l-lg bg-white pr-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon && <Icon name={icon} />}
        {title}
      </div>
      {isActive && <OutsideCorner rightTop={!isFirst} rightBottom={true} />}
    </Link>
  );
}

export function SidebarLayout() {
  const machines = useMachines();
  const items: SidebarItemContent[] = [
    ...machines.map((machine) => ({
      link: `/_sidebar/machines/${machine.slug}/${machine.machine_identification.serial}/control`,
      activeLink: `/_sidebar/machines/${machine.slug}/${machine.machine_identification.serial}`,
      title: machine.name,
      icon: machine.icon,
    })),
    {
      link: "/_sidebar/setup/ethercat",
      activeLink: "/_sidebar/setup",
      title: "Setup",
      icon: "lu:Settings2",
    },
  ];

  return (
    <div className="flex h-full flex-row">
      <div className="w-48 min-w-48" />
      <div className="fixed flex h-full w-48 flex-col bg-neutral-200">
        <div className="flex h-20 flex-col items-center justify-center gap-0 pt-2">
          <div className="font-qitech line-clamp-none text-3xl">QITECH</div>
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
