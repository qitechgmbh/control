import React, { useState } from "react";
import { Button } from "../ui/button";
import { Icon } from "../Icon";
import { cn } from "@/lib/utils";

/**
 * Virtual keyboard component for touch devices.
 * Provides a QWERTZ layout keyboard that can be used to input text
 * into any input field in the application.
 */
export function VirtualKeyboard({
  onKeyPress,
  onClose,
  inputType = "text",
}: {
  onKeyPress: (key: string) => void;
  onClose?: () => void;
  inputType?: "text" | "number" | "email" | "tel";
}) {
  const [isShift, setIsShift] = useState(false);
  const [isSymbols, setIsSymbols] = useState(false);

  // QWERTZ layout (German keyboard)
  const keysRow1 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
  const keysRow2 = ["q", "w", "e", "r", "t", "z", "u", "i", "o", "p"];
  const keysRow3 = ["a", "s", "d", "f", "g", "h", "j", "k", "l"];
  const keysRow4 = ["y", "x", "c", "v", "b", "n", "m"];

  // Symbol layout
  const symbolsRow1 = ["!", "@", "#", "$", "%", "^", "&", "*", "(", ")"];
  const symbolsRow2 = ["-", "_", "+", "=", "[", "]", "{", "}", "|", "\\"];
  const symbolsRow3 = [";", ":", "'", '"', ",", ".", "<", ">", "/", "?"];
  const symbolsRow4 = ["~", "`", "€", "£", "¥", "§", "°", "±"];

  const handleKeyPress = (key: string) => {
    const finalKey = isShift ? key.toUpperCase() : key;
    onKeyPress(finalKey);
    // Auto-disable shift after one key press
    if (isShift) {
      setIsShift(false);
    }
  };

  const handleShift = () => {
    setIsShift(!isShift);
  };

  const handleSymbols = () => {
    setIsSymbols(!isSymbols);
    setIsShift(false); // Disable shift when switching to symbols
  };

  // For numeric inputs, show a simplified numpad instead
  if (inputType === "number" || inputType === "tel") {
    return (
      <div 
        className="fixed bottom-0 left-0 right-0 z-[9999] border-t bg-background p-4 shadow-2xl"
        data-virtual-keyboard
      >
        <div className="mx-auto grid max-w-md grid-cols-3 gap-3">
          {["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", ".", "-"].map(
            (key) => (
              <Button
                key={key}
                variant="outline"
                className="h-16 text-2xl font-normal"
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                }}
                onClick={() => handleKeyPress(key)}
              >
                {key}
              </Button>
            ),
          )}
          <Button
            variant="outline"
            className="col-span-2 h-16 text-lg"
            onMouseDown={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onClick={() => onKeyPress("BACKSPACE")}
          >
            <Icon name="lu:Delete" className="mr-2" />
            Delete
          </Button>
          {onClose && (
            <Button
              variant="outline"
              className="h-16 text-lg"
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={onClose}
            >
              <Icon name="lu:X" className="mr-2" />
              Close
            </Button>
          )}
        </div>
      </div>
    );
  }

  // Full QWERTZ keyboard for text inputs
  const currentRow1 = isSymbols ? symbolsRow1 : keysRow1;
  const currentRow2 = isSymbols ? symbolsRow2 : keysRow2;
  const currentRow3 = isSymbols ? symbolsRow3 : keysRow3;
  const currentRow4 = isSymbols ? symbolsRow4 : keysRow4;

  return (
    <div 
      className="fixed bottom-0 left-0 right-0 z-[9999] border-t bg-background p-3 shadow-2xl"
      data-virtual-keyboard
    >
      <div className="mx-auto max-w-4xl">
        {/* Row 1: Numbers/Symbols */}
        <div className="mb-2 flex gap-1.5">
          {currentRow1.map((key) => (
            <Button
              key={key}
              variant="outline"
              className={cn(
                "h-12 flex-1 text-base font-normal",
                isShift && !isSymbols && "bg-primary/20",
              )}
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={() => handleKeyPress(key)}
            >
              {isShift && !isSymbols ? key.toUpperCase() : key}
            </Button>
          ))}
        </div>

        {/* Row 2 */}
        <div className="mb-2 flex gap-1.5">
          {currentRow2.map((key) => (
            <Button
              key={key}
              variant="outline"
              className={cn(
                "h-12 flex-1 text-base font-normal",
                isShift && !isSymbols && "bg-primary/20",
              )}
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={() => handleKeyPress(key)}
            >
              {isShift && !isSymbols ? key.toUpperCase() : key}
            </Button>
          ))}
        </div>

        {/* Row 3 */}
        <div className="mb-2 flex gap-1.5">
          {currentRow3.map((key) => (
            <Button
              key={key}
              variant="outline"
              className={cn(
                "h-12 flex-1 text-base font-normal",
                isShift && !isSymbols && "bg-primary/20",
              )}
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={() => handleKeyPress(key)}
            >
              {isShift && !isSymbols ? key.toUpperCase() : key}
            </Button>
          ))}
        </div>

        {/* Row 4: Bottom row with special keys */}
        <div className="flex gap-1.5">
          <Button
            variant={isShift ? "default" : "outline"}
            className="h-12 px-4 text-sm font-medium"
            onMouseDown={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onClick={handleShift}
          >
            <Icon name="lu:ArrowUp" />
          </Button>
          {currentRow4.map((key) => (
            <Button
              key={key}
              variant="outline"
              className={cn(
                "h-12 flex-1 text-base font-normal",
                isShift && !isSymbols && "bg-primary/20",
              )}
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={() => handleKeyPress(key)}
            >
              {isShift && !isSymbols ? key.toUpperCase() : key}
            </Button>
          ))}
          <Button
            variant="outline"
            className="h-12 flex-1 text-sm font-medium"
            onMouseDown={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onClick={() => handleKeyPress(" ")}
          >
            Space
          </Button>
          <Button
            variant="outline"
            className="h-12 px-4 text-sm font-medium"
            onMouseDown={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onClick={() => onKeyPress("BACKSPACE")}
          >
            <Icon name="lu:Delete" />
          </Button>
          {onClose && (
            <Button
              variant="outline"
              className="h-12 px-4 text-sm font-medium"
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
              onClick={onClose}
            >
              <Icon name="lu:X" />
            </Button>
          )}
        </div>

        {/* Symbol toggle */}
        <div className="mt-2 flex justify-center">
          <Button
            variant={isSymbols ? "default" : "outline"}
            size="sm"
            className="h-8 text-xs"
            onMouseDown={(e) => {
              e.preventDefault();
              e.stopPropagation();
            }}
            onClick={handleSymbols}
          >
            {isSymbols ? "ABC" : "123"}
          </Button>
        </div>
      </div>
    </div>
  );
}

