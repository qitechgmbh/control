import { Icon, IconName } from "@/components/Icon";
import { Badge } from "@/components/ui/badge";
import { cva } from "class-variance-authority";
import React from "react";

type Props = {
  variant: "error" | "success" | "info";
  children: React.ReactNode;
};

export function StatusBadge({ variant, children }: Props) {
  const badgeStyle = cva(["text-md"], {
    variants: {
      variant: {
        error: "bg-red-500",
        success: "bg-green-600",
        info: "bg-blue-500",
      },
    },
  });
  const icon: IconName =
    variant === "error"
      ? "lu:TriangleAlert"
      : variant === "success"
        ? "lu:Check"
        : "lu:Info";
  return (
    <Badge
      className={badgeStyle({
        variant,
      })}
    >
      <Icon name={icon} className="size-6" />
      {children}
    </Badge>
  );
}
