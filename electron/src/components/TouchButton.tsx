import { useClassNameBuilder } from "@/helpers/style";
import { Button } from "./ui/button";
import React, { ComponentProps } from "react";
import { Icon, IconName } from "./Icon";

type Props = {
  icon?: IconName;
  isLoading?: boolean;
} & ComponentProps<typeof Button>;

export function TouchButton({
  children,
  icon,
  className,
  isLoading,
  disabled,
  ...props
}: Props) {
  const buttonStyle = useClassNameBuilder({
    base: "px-8 py-10 text-md",
  });
  return (
    <Button
      className={buttonStyle({ className })}
      disabled={isLoading || disabled}
      {...props}
    >
      <div className="flex flex-row items-center gap-2">
        {icon && !isLoading && <Icon name={icon} />}
        {isLoading && <Icon name="lu:Loader" className="animate-spin" />}
        {children}
      </div>
    </Button>
  );
}
