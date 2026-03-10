import React, { ComponentProps } from "react";
import { Button } from "../ui/button";
import { Icon, IconName } from "../Icon";
import { LoadingSpinner } from "../LoadingSpinner";
import { cva } from "class-variance-authority";

type Props = {
  enabled: boolean;
  onToggle: (next: boolean) => void;
  label: string;
  iconOn?: IconName;
  iconOff?: IconName;
  isLoading?: boolean;
} & Omit<ComponentProps<typeof Button>, "onClick">;

export function ToggleButton({
  enabled,
  onToggle,
  label,
  iconOn,
  iconOff,
  isLoading,
  className,
  disabled,
  ...props
}: Props) {
  const buttonStyle = cva("px-6 py-6 text-md h-max", {
    variants: {
      enabled: {
        true: "bg-green-600 hover:bg-green-700 text-white",
        false: "bg-muted hover:bg-muted/80 text-muted-foreground",
      },
    },
  });

  const icon = enabled ? iconOn : iconOff;

  return (
    <Button
      className={buttonStyle({ enabled, className })}
      disabled={isLoading || disabled}
      onClick={() => onToggle(!enabled)}
      {...props}
    >
      <div className="flex flex-row items-center gap-2 text-wrap">
        {isLoading ? (
          <LoadingSpinner />
        ) : (
          icon && <Icon name={icon} className="size-6" />
        )}
        {label}
        <span className="rounded-full bg-black/10 px-2 py-0.5 text-sm font-semibold">
          {enabled ? "ON" : "OFF"}
        </span>
      </div>
    </Button>
  );
}
