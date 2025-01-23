"use client";
import { usePathname, useRouter } from "next/navigation";
import React from "react";
import { OutsideCorner } from "./OutsideCorner";

type Item = {
  icon?: React.ReactNode;
  title: string;
  link?: string;
  isFirst?: boolean;
};

type Props = {
  items: Item[];
};

function TopbarItem({ icon, title, isFirst, link }: Item) {
  const router = useRouter();
  const pathname = usePathname();
  const isActive = link ? pathname.startsWith(link) : false;

  return (
    <div
      className={`h-full relative ${isActive ? "" : "pb-2"}`}
      onClick={() => (link && !isActive ? router.push(link) : null)}
    >
      <div
        className={`flex items-center justify-center h-full gap-2 text-md px-4 z-10 relative ${
          isActive ? "bg-white rounded-t-lg pb-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon}
        {title}
      </div>
      <OutsideCorner bottomLeft={isActive && !isFirst} bottomRight={isActive} />
    </div>
  );
}

export function Topbar({ items }: Props) {
  return (
    <>
      <div
        className="h-20 w-full bg-neutral-200 fixed flex gap-2 pt-2"
        style={{ zIndex: 1000 }}
      >
        {items.map((item, index) => (
          <TopbarItem key={index} {...item} isFirst={index == 0} />
        ))}
      </div>
      <div className="h-20"></div>
    </>
  );
}
