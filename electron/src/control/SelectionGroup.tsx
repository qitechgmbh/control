import React, { ComponentProps } from "react";
import { TouchButton } from "@/components/touch/TouchButton";

export type Props<KEY extends string> = {
  value?: KEY | undefined;
  onChange?: (value: KEY) => void;
  disabled?: boolean;
  loading?: boolean;
  options: {
    [K in KEY]: Option;
  };
  orientation?: "horizontal" | "vertical";
  className?: string;
};

type Option = ComponentProps<typeof TouchButton> & {
  isActiveClassName?: string;
  disabled?: boolean;
  className?: string;
  /** Optional confirmation message. If provided, shows a native browser confirmation dialog before executing onChange callback */
  confirmation?: string;
};

export function SelectionGroup<KEY extends string>({
  options,
  onChange,
  disabled,
  loading,
  value,
  className = "",
}: Props<KEY>) {
  return (
    <div className={`flex flex-row flex-wrap gap-4 ${className}`}>
      {Object.entries(options as Record<string, Option>).map(
        ([
          key,
          {
            children,
            icon,
            className,
            isActiveClassName,
            disabled: optionDisabled,
            confirmation,
          },
        ]) => (
          <TouchButton
            key={key}
            icon={icon}
            disabled={disabled || optionDisabled}
            onClick={() => {
              // Check if this option requires confirmation
              if (confirmation) {
                // Show native browser confirmation dialog
                if (window.confirm(confirmation)) {
                  // User clicked "OK" - proceed with the action
                  onChange?.(key as KEY);
                }
                // If user clicked "Cancel", do nothing
              } else {
                // No confirmation required - execute action immediately
                onChange?.(key as KEY);
              }
            }}
            variant={key === value ? "default" : "outline"}
            isLoading={key === value && loading}
            className={
              className + (key === value ? ` ${isActiveClassName}` : "")
            }
          >
            {children}
          </TouchButton>
        ),
      )}
    </div>
  );
}

type SelectionGroupBooleanProps = {
  value?: boolean;
  onChange?: (value: boolean) => void;
  disabled?: boolean;
  loading?: boolean;
  optionTrue: Option;
  optionFalse: Option;
  reverse?: boolean;
};

export function SelectionGroupBoolean({
  value,
  onChange,
  disabled,
  loading,
  optionTrue,
  optionFalse,
  reverse,
}: SelectionGroupBooleanProps) {
  return (
    <SelectionGroup<"on" | "off">
      value={value ? "on" : "off"}
      disabled={disabled}
      loading={loading}
      options={
        reverse
          ? {
              on: optionFalse,
              off: optionTrue,
            }
          : {
              off: optionFalse,
              on: optionTrue,
            }
      }
      onChange={(value) => onChange?.(value === "on")}
    />
  );
}
