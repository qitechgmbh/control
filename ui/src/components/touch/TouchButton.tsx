import { Button } from "../ui/button";
import React, { ComponentProps } from "react";
import { Icon, IconName } from "../Icon";
import { cva } from "class-variance-authority";
import { LoadingSpinner } from "../LoadingSpinner";

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
  const buttonStyle = cva("px-6 py-6 text-md h-max");
  return (
    <Button
      className={buttonStyle({ className })}
      disabled={isLoading || disabled}
      {...props}
    >
      <div className="flex flex-row items-center gap-2 text-wrap">
        {icon && !isLoading && <Icon name={icon} className="size-6" />}
        {isLoading && <LoadingSpinner />}
        {children}
      </div>
    </Button>
  );
}
