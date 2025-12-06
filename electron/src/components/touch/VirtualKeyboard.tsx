import React from "react";
import { Button } from "../ui/button";
import { Icon } from "../Icon";
import { TouchNumpad } from "./TouchNumpad";

/**
 * Virtual keyboard component for touch devices.
 * Provides a numpad for numeric input fields.
 */
export function VirtualKeyboard({
  onClose,
  inputType = "text",
  onNumpadDigit,
  onNumpadDecimal,
  onNumpadDelete,
  onNumpadToggleSign,
  onNumpadCursorLeft,
  onNumpadCursorRight,
}: {
  onClose?: () => void;
  inputType?: "text" | "number" | "email" | "tel";
  onNumpadDigit?: (digit: string) => void;
  onNumpadDecimal?: () => void;
  onNumpadDelete?: () => void;
  onNumpadToggleSign?: () => void;
  onNumpadCursorLeft?: () => void;
  onNumpadCursorRight?: () => void;
}) {
  // Only show numpad for numeric inputs
  if (inputType !== "number" && inputType !== "tel") {
    return null;
  }

  // Debug: Log when numpad should be shown
  console.log("VirtualKeyboard rendering numpad for inputType:", inputType);

  return (
    <div 
      className="fixed bottom-0 left-0 right-0 z-[9999] border-t bg-background p-4 shadow-2xl"
      data-virtual-keyboard
      data-virtual-keyboard-root="true"
    >
      <div className="mx-auto flex max-w-md flex-col gap-4">
        <div className="flex justify-end">
          {onClose && (
            <Button
              variant="outline"
              className="h-12 text-lg"
              tabIndex={-1}
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                onClose();
              }}
            >
              <Icon name="lu:X" className="mr-2" />
              Close
            </Button>
          )}
        </div>
        <div className="flex justify-center">
          <TouchNumpad
            onDigit={onNumpadDigit}
            onDecimal={onNumpadDecimal}
            onDelete={onNumpadDelete}
            onToggleSign={onNumpadToggleSign}
            onCursorLeft={onNumpadCursorLeft}
            onCursorRight={onNumpadCursorRight}
          />
        </div>
      </div>
    </div>
  );
}

