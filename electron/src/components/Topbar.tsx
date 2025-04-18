"use client";
import React, { Fragment } from "react";
import { OutsideCorner } from "./OutsideCorner";
import { BackButton } from "./BackButton";
import { Link, Outlet } from "@tanstack/react-router";
import { useOnSubpath } from "@/lib/useOnSubpath";
import { Icon, IconName } from "./Icon";

type TopbarItemContent = {
  link: string;
  icon?: IconName;
  title: string;
};

type TopbarItemProps = TopbarItemContent;
export function TopbarItem({ icon, title, link }: TopbarItemProps) {
  const isActive = useOnSubpath(link);
  return (
    <Link className={`relative h-full ${isActive ? "" : "pb-2"}`} to={link}>
      <div
        className={`text-md relative z-10 flex h-full items-center justify-center gap-2 px-6 ${
          isActive ? "rounded-t-lg bg-white pb-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon && <Icon name={icon} />}
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
      <div className="h-18"></div>
      <div
        className="fixed flex h-18 w-full gap-2 bg-neutral-200 pt-2"
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
