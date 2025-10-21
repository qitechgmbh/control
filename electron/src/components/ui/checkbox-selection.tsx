import * as React from "react";
import { cn } from "@/lib/utils";

interface CheckboxProps extends React.ComponentProps<"input"> {
  label?: string; // optional label next to the checkbox
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, ...props }, ref) => {
    return (
      <label
        className={cn("inline-flex cursor-pointer items-center space-x-2")}
      >
        <input
          type="checkbox"
          ref={ref}
          className={cn(
            "border-input bg-background text-primary focus-visible:ring-ring h-4 w-4 rounded border focus-visible:ring-2 focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50",
            className,
          )}
          {...props}
        />
        {label && <span className="text-foreground text-sm">{label}</span>}
      </label>
    );
  },
);

Checkbox.displayName = "Checkbox";

export { Checkbox };
