import { Icon, IconName } from "@/components/Icon";
import { Badge } from "@/components/ui/badge";
import { cva } from "class-variance-authority";
import React from "react";

type Props = {
  variant: "error" | "warning" | "success";
  children: React.ReactNode;
};

export function StatusBadge({ variant, children }: Props) {
  const badgeStyle = cva(["text-md"], {
    variants: {
      variant: {
        error: "bg-red-500",
        warning: "bg-yellow-500",
        success: "bg-green-600",
      },
    },
  });
  const icon: IconName =
    variant === "success" ? "lu:Check" : "lu:TriangleAlert";
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
