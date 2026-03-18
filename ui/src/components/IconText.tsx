import React from "react";
import { Icon, IconName } from "./Icon";
import { cva } from "class-variance-authority";

export type Props = {
  icon: IconName;
  children?: React.ReactNode;
  variant?: "info" | "warning" | "error" | "success";
};

export function IconText({ icon, children, variant = "info" }: Props) {
  const divStyle = cva(["flex", "flex-row", "items-center", "gap-2"], {
    variants: {
      variant: {
        info: "text-blue-500",
        warning: "text-amber-500",
        error: "text-red-500",
        success: "text-green-500",
      },
    },
  });
  return (
    <div
      className={divStyle({
        variant,
      })}
    >
      <Icon name={icon} />
      {children && <div>{children}</div>}
    </div>
  );
}
