import * as React from "react";

import { cn } from "@/lib/utils";

function Input({ className, type, inputMode, ...props }: React.ComponentProps<"input">) {
  // Automatically determine inputMode based on type if not explicitly provided
  // This ensures the system keyboard (GNOME on-screen keyboard) appears
  const autoInputMode = React.useMemo(() => {
    if (inputMode) return inputMode;
    
    // Determine inputMode based on type
    switch (type) {
      case "number":
      case "tel":
        return "numeric";
      case "email":
        return "email";
      case "url":
        return "url";
      case "search":
        return "search";
      default:
        return "text";
    }
  }, [type, inputMode]);

  return (
    <input
      type={type}
      inputMode={autoInputMode}
      data-slot="input"
      className={cn(
        "border-input file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
        "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
        "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
        className,
      )}
      {...props}
    />
  );
}

export { Input };
