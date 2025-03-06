"use client";
import React, { Fragment } from "react";
import { OutsideCorner } from "./OutsideCorner";
import { BackButton } from "./BackButton";
import { Link, Outlet } from "@tanstack/react-router";
import { useOnSubpath } from "@/hooks/useOnSubpath";

// type TopbarItemProps = {
//   icon?: React.ReactNode;
//   title: string;
// } & SidebarButtonBuilderProps;

// export type SidebarButtonBuilderProps = {
//   isActive: boolean;
//   isFirst: boolean;
//   onClick: () => void;
// };

type TopbarItemContent = {
  link: string;
  icon?: React.ReactNode;
  title: string;
};

type TopbarItemProps = TopbarItemContent;
export function TopbarItem({ icon, title, link }: TopbarItemProps) {
  const isActive = useOnSubpath(link);
  return (
    <Link className={`relative h-full ${isActive ? "" : "pb-2"}`} to={link}>
      <div
        className={`text-md relative z-10 flex h-full items-center justify-center gap-2 px-4 ${
          isActive ? "rounded-t-lg bg-white pb-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon}
        {title}
      </div>
      <OutsideCorner bottomLeft={isActive} bottomRight={isActive} />
    </Link>
  );
}

type TopbarProps = {
  items: TopbarItemContent[];
  pathname: string;
};

export function Topbar({ items, pathname }: TopbarProps) {
  return (
    <div className="flex h-full flex-col">
      <div className="h-20"></div>
      <div
        className="fixed flex h-20 w-full gap-2 bg-neutral-200 pt-2"
        // 50 is below popup dialogs
        style={{ zIndex: 50 }}
      >
        <div className="flexflex-col z-10 pb-2">
          <BackButton />
        </div>
        {items.map((item, index) => {
          let link = item.link;
          if (!item.link.startsWith("/")) {
            link = pathname + "/" + item.link;
          }
          return (
            <Fragment key={index}>
              <TopbarItem {...item} link={link} />
            </Fragment>
          );
        })}
      </div>
      <Outlet />
    </div>
  );
}
