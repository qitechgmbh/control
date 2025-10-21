import * as React from "react";
import { CheckIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface CheckboxProps extends React.ComponentProps<"input"> {
  label?: string;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, ...props }, ref) => {
    return (
      <label className="inline-flex items-center gap-2 cursor-pointer text-sm text-foreground">
        <span className="relative flex h-4 w-4 items-center justify-center">
          <input
            ref={ref}
            type="checkbox"
            className={cn(
              "peer appearance-none h-4 w-4 rounded border border-input bg-background shadow-xs transition-colors",
              "focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50",
              "checked:bg-primary checked:border-primary",
              className
            )}
            {...props}
          />
          <CheckIcon className="absolute size-3 text-primary-foreground opacity-0 peer-checked:opacity-100 transition-opacity" />
        </span>
        {label && <span>{label}</span>}
      </label>
    );
  }
);

Checkbox.displayName = "Checkbox";

export { Checkbox };

