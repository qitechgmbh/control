import { cva } from "class-variance-authority";
import React from "react";

type Props = {
  height?: 1 | 2 | 3;
  width?: 1 | 2 | 3;
  children?: React.ReactNode;
  className?: string;
  title?: string;
};

export function ControlCard({
  height,
  width,
  children,
  className,
  title,
}: Props) {
  const cardStyle = cva(
    [
      "bg-white",
      "border",
      "border-gray-200",
      "rounded-3xl",
      "p-6",
      "pt-4",
      "flex-1",
      "win-w-1/3",
      "flex",
      "flex-col",
      "gap-4",
      "shadow",
    ],
    {
      variants: {
        height: {
          1: "row-span-1",
          2: "row-span-2",
          3: "row-span-3",
        },
        width: {
          1: "col-span-1",
          2: "col-span-2",
          3: "col-span-3",
        },
      },
    },
  );

  return (
    <div
      className={cardStyle({
        height,
        width,
        className,
      })}
    >
      {title && <h2 className="text-2xl font-bold">{title}</h2>}
      {children}
    </div>
  );
}
