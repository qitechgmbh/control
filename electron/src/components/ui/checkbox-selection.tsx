import * as React from "react";
import { CheckIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface CheckboxProps extends React.ComponentProps<"input"> {
  label?: string;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, ...props }, ref) => {
    return (
      <label className="text-foreground inline-flex cursor-pointer items-center gap-2 text-sm">
        <span className="relative flex h-4 w-4 items-center justify-center">
          <input
            ref={ref}
            type="checkbox"
            className={cn(
              "peer border-input bg-background h-4 w-4 appearance-none rounded border shadow-xs transition-colors",
              "focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-none",
              "checked:bg-primary checked:border-primary",
              className,
            )}
            {...props}
          />
          <CheckIcon className="text-primary-foreground absolute size-3 opacity-0 transition-opacity peer-checked:opacity-100" />
        </span>
        {label && <span>{label}</span>}
      </label>
    );
  },
);

Checkbox.displayName = "Checkbox";

export { Checkbox };
