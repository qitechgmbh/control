import React from "react";
import { Button } from "../ui/button";
import { Icon } from "../Icon";

export type TouchKeyboardProps = {
  onKey?: (char: string) => void;
  onSpace?: () => void;
  onDelete?: () => void;
  onClear?: () => void;
  onCursorLeft?: () => void;
  onCursorRight?: () => void;
};

const KEYBOARD_ROWS = ["1234567890", "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

export function TouchKeyboard({
  onKey,
  onSpace,
  onDelete,
  onClear,
  onCursorLeft,
  onCursorRight,
}: TouchKeyboardProps = {}) {
  return (
    <div className="grid h-full w-max grid-cols-10 gap-4">
      {KEYBOARD_ROWS.map((row) =>
        row.split("").map((char) => (
          <TouchKeyboardButton key={`${row}-${char}`} onClick={() => onKey?.(char)}>
            {char}
          </TouchKeyboardButton>
        )),
      )}

      <TouchKeyboardButton className="col-span-2" onClick={() => onSpace?.()}>
        Space
      </TouchKeyboardButton>
      <TouchKeyboardButton onClick={() => onDelete?.()}>
        <Icon name="lu:Delete" />
      </TouchKeyboardButton>
      <TouchKeyboardButton onClick={() => onClear?.()}>Clear</TouchKeyboardButton>
      <TouchKeyboardButton onClick={() => onCursorLeft?.()}>
        <Icon name="lu:ArrowLeft" />
      </TouchKeyboardButton>
      <TouchKeyboardButton onClick={() => onCursorRight?.()}>
        <Icon name="lu:ArrowRight" />
      </TouchKeyboardButton>
    </div>
  );
}

type TouchKeyboardButtonProps = {
  children: React.ReactNode;
  onClick: () => void;
  className?: string;
};

function TouchKeyboardButton({
  children,
  onClick,
  className,
}: TouchKeyboardButtonProps) {
  const width = className?.includes("col-span-2") ? "w-full" : "w-22";
  return (
    <Button
      className={`h-full ${width} font-mono text-2xl font-normal ${className || ""}`}
      variant="outline"
      onMouseDown={(e) => {
        // Keep focus on the associated text input while tapping keys.
        e.preventDefault();
      }}
      onClick={onClick}
    >
      {children}
    </Button>
  );
}
