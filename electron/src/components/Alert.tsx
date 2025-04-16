import {
  Alert as UIAlert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import React from "react";
import { useClassNameBuilder } from "@/helpers/style";
import { Icon, IconName, IconNameMap } from "./Icon";

type Props = {
  title: string;
  children: React.ReactNode;
  variant?: "info" | "warning" | "error";
  className?: string;
  icon?: IconName;
};

export function Alert({
  variant = "info",
  title,
  children,
  className,
  icon,
}: Props) {
  const alertStyle = useClassNameBuilder({
    base: "p-4 flex flex-col gap-2 justify-center",
    variables: {
      variant: {
        info: "border-blue-300  bg-blue-50",
        warning: "border-amber-300 bg-amber-50",
        error: "border-red-300 bg-red-50",
      },
    },
  });

  const iconStyle = useClassNameBuilder({
    base: "size-4",
    variables: {
      variant: {
        info: "text-blue-500",
        warning: "text-amber-500",
        error: "text-red-500",
      },
    },
  });

  const headerStyle = useClassNameBuilder({
    base: "flex flex-row gap-2 items-center",
    variables: {
      variant: {
        info: "text-blue-500",
        warning: "text-amber-500",
        error: "text-red-500",
      },
    },
  });

  const icons: IconNameMap = {
    info: "lu:Info",
    warning: "lu:CircleAlert",
    error: "lu:TriangleAlert",
  };

  const _icon = icon || icons[variant];

  return (
    <UIAlert className={alertStyle({ variant, className })}>
      <AlertTitle
        className={headerStyle({
          variant,
        })}
      >
        <Icon name={_icon} className={iconStyle({ variant })} />
        {title}
      </AlertTitle>
      <AlertDescription>{children}</AlertDescription>
    </UIAlert>
  );
}
