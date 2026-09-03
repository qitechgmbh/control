import { cn } from "@/lib/utils";
import React, { useState } from "react";

type KeyboardLayout = string[][];

const MAIN_LAYOUT: KeyboardLayout = [
  ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "⌫"],
  ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
  ["a", "s", "d", "f", "g", "h", "j", "k", "l", "↵"],
  ["⇧", "z", "x", "c", "v", "b", "n", "m", ",", ".", "⇧"],
  ["⎵"],
];

const SHIFT_LAYOUT: KeyboardLayout = [
  ["!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "⌫"],
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L", "↵"],
  ["⇧", "Z", "X", "C", "V", "B", "N", "M", "<", ">", "⇧"],
  ["⎵"],
];

interface TouchKeyboardProps {
  value: string;
  onChange: (value: string) => void;
  onEnter?: () => void;
  visible: boolean;
  className?: string;
}

const SPECIAL_KEYS = new Set(["⌫", "↵", "⇧", "⎵"]);

const KEY_WIDTH: Record<string, string> = {
  "⌫": "flex-[1.4]",
  "↵": "flex-[1.4]",
  "⇧": "flex-[1.3]",
  "⎵": "flex-[5]",
};

const TouchKeyboard: React.FC<TouchKeyboardProps> = ({
  value,
  onChange,
  onEnter,
  visible,
  className,
}) => {
  const [shift, setShift] = useState(false);

  const layout = shift ? SHIFT_LAYOUT : MAIN_LAYOUT;

  const handleKey = (key: string) => {
    if (key === "⌫") {
      onChange(value.slice(0, -1));
      return;
    }

    if (key === "↵") {
      onEnter?.();
      return;
    }

    if (key === "⇧") {
      setShift((currentShift) => !currentShift);
      return;
    }

    if (key === "⎵") {
      onChange(value + " ");
      setShift(false);
      return;
    }

    onChange(value + key);

    if (shift) {
      setShift(false);
    }
  };

  if (!visible) return null;

  return (
    <div
      className={cn(
        "border-border bg-card flex w-full flex-col gap-2 rounded-xl border p-4 shadow-lg select-none",
        className,
      )}
    >
      {layout.map((row, rowIndex) => (
        <div key={rowIndex} className="flex w-full justify-center gap-1.5">
          {row.map((key, keyIndex) => {
            const isSpecial = SPECIAL_KEYS.has(key);
            const widthClass = KEY_WIDTH[key] ?? "flex-1";
            const isShiftActive = key === "⇧" && shift;

            return (
              <button
                key={`${rowIndex}-${keyIndex}-${key}`}
                type="button"
                onPointerDown={(e) => {
                  e.preventDefault();
                  handleKey(key);
                }}
                className={cn(
                  "h-14 min-w-0 rounded-md border px-2 text-base font-medium transition-transform duration-75 focus:outline-none active:scale-95",
                  widthClass,
                  isSpecial
                    ? isShiftActive
                      ? "border-sky-500 bg-sky-500 text-white"
                      : "border-muted bg-muted hover:bg-muted/80"
                    : "border-border bg-background hover:bg-muted/50",
                )}
              >
                {key}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
};

export { TouchKeyboard };
export default TouchKeyboard;
