import React, { ComponentProps } from "react";
import { TouchButton } from "@/components/TouchButton";

export type Props<KEY extends string> = {
  value?: KEY | undefined;
  onChange?: (value: KEY) => void;
  options: {
    [K in KEY]: Option;
  };
  orientation?: "horizontal" | "vertical";
};

type Option = ComponentProps<typeof TouchButton> & {
  isActiveClassName?: string;
};

export function SelectionGroup<KEY extends string>({
  options,
  onChange,
  value,
}: Props<KEY>) {
  return (
    <div className="flex flex-row flex-wrap gap-4">
      {Object.entries(options as Record<string, Option>).map(
        ([key, { children, icon, className, isActiveClassName }]) => (
          <TouchButton
            key={key}
            icon={icon}
            onClick={() => onChange?.(key as KEY)}
            variant={key === value ? "default" : "outline"}
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
  optionTrue: Option;
  optionFalse: Option;
};

export function SelectionGroupBoolean({
  value,
  onChange,
  optionTrue,
  optionFalse,
}: SelectionGroupBooleanProps) {
  return (
    <SelectionGroup<"on" | "off">
      value={value ? "on" : "off"}
      options={{
        off: optionFalse,
        on: optionTrue,
      }}
      onChange={(value) => onChange?.(value === "on")}
    />
  );
}
