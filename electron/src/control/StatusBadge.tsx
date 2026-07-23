import { Icon, IconName } from "@/components/Icon";
import { Badge } from "@/components/ui/badge";
import { cva } from "class-variance-authority";
import React from "react";

type Props = {
  variant: "error" | "success" | "warning";
  children: React.ReactNode;
};

export function StatusBadge({ variant, children }: Props) {
  const badgeStyle = cva(["text-md", "max-w-full", "whitespace-normal"], {
    variants: {
      variant: {
        error: "bg-red-500",
        success: "bg-green-600",
        warning: "bg-amber-500",
      },
    },
  });
  const icon: IconName =
    variant === "error"
      ? "lu:TriangleAlert"
      : variant === "warning"
        ? "lu:Clock"
        : "lu:Check";
  return (
    <Badge
      className={badgeStyle({
        variant,
      })}
    >
      <Icon name={icon} className="size-6 shrink-0" />
      {children}
    </Badge>
  );
}
