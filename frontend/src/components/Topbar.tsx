"use client";
import React, { Fragment } from "react";
import { OutsideCorner } from "./OutsideCorner";
import { BackButton } from "./BackButton";

type TopbarItemProps = {
  icon?: React.ReactNode;
  title: string;
} & SidebarButtonBuilderProps;

export type SidebarButtonBuilderProps = {
  isActive: boolean;
  isFirst: boolean;
  onClick: () => void;
};

export function TopbarItem({
  icon,
  title,
  isActive,
  onClick,
}: TopbarItemProps) {
  return (
    <div
      className={`h-full relative ${isActive ? "" : "pb-2"}`}
      onClick={onClick}
    >
      <div
        className={`flex items-center justify-center h-full gap-2 text-md px-4 z-10 relative ${
          isActive ? "bg-white rounded-t-lg pb-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon}
        {title}
      </div>
      <OutsideCorner bottomLeft={isActive} bottomRight={isActive} />
    </div>
  );
}

type TopbarProps = {
  items: {
    button: (props: SidebarButtonBuilderProps) => React.ReactNode;
    children: React.ReactNode;
  }[];
  initialActiveIndex: number;
};

export function Topbar({ items, initialActiveIndex }: TopbarProps) {
  const [activeIndex, setActiveIndex] = React.useState<number | null>(
    initialActiveIndex
  );

  return (
    <div className="flex flex-col h-full">
      <div className="h-20"></div>
      <div
        className="h-20 w-full bg-neutral-200 fixed flex gap-2 pt-2"
        style={{ zIndex: 1000 }}
      >
        <div className="h-full flex flex-col justify-end pb-2 z-10">
          <BackButton />
        </div>
        {items.map((item, index) => (
          <Fragment key={index}>
            {item.button({
              isActive: activeIndex === index,
              isFirst: index === 0,
              onClick: () => {
                console.log("clicked", index);
                setActiveIndex(index);
              },
            })}
          </Fragment>
        ))}
      </div>
      {items.find((_, index) => activeIndex === index)?.children}
    </div>
  );
}
