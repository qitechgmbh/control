"use client";
import { useState } from "react";
import { OutsideCorner } from "./OutsideCorner";

type SidbarItemProps = {
  icon: React.ReactNode;
  children: React.ReactNode;
  // used for active state detection
  parentLink?: string;
  // actual navigation link
  link?: string;
} & SidebarButtonBuilderProps;

export type SidebarButtonBuilderProps = {
  isActive: boolean;
  isFirst: boolean;
  onClick: () => void;
};

export function SidbarItem({
  icon,
  children,
  isFirst,
  isActive,
  onClick,
}: SidbarItemProps) {
  return (
    <div
      className={`w-full h-20 relative ${isActive ? " pl-2" : " px-2"}`}
      onClick={onClick}
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

type SidebarProps = {
  items: {
    button: (props: SidebarButtonBuilderProps) => React.ReactNode;
    children: React.ReactNode;
  }[];
  initialActiveIndex: number;
};

export function Sidebar({ items, initialActiveIndex }: SidebarProps) {
  const [activeIndex, setActiveIndex] = useState<number | null>(
    initialActiveIndex
  );

  return (
    <div className="flex flex-row h-full">
      <div className="w-40" />
      <div className="flex flex-col h-full fixed w-40 bg-neutral-200">
        <div className="h-20">QiTech Control</div>
        <div className="flex flex-col gap-2">
          {items.map((item, index) => (
            <div key={index}>
              {item.button({
                isActive: activeIndex === index,
                isFirst: index === 0,
                onClick: () => {
                  console.log("clicked", index);
                  setActiveIndex(index);
                },
              })}
            </div>
          ))}
        </div>
      </div>
      {items.find((item, index) => activeIndex === index)?.children}
    </div>
  );
}
