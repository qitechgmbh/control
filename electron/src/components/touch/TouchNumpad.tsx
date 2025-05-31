import React from "react";
import { Popover, PopoverContent } from "../ui/popover";
import { Button } from "../ui/button";
import { Icon } from "../Icon";

export type TouchNumpadProps = {
  isOpen: boolean;
  setIsOpen?: (isOpen: boolean) => void;
};

export function TouchNumpadPopover({ isOpen, setIsOpen }: TouchNumpadProps) {
  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverContent className="mx-8 flex w-min flex-col gap-6 rounded-2xl p-6 shadow-2xl">
        <TouchNumpad />
      </PopoverContent>
    </Popover>
  );
}

export type TouchNumpadInputProps = {
  onDigit?: (digit: string) => void;
  onDecimal?: () => void;
  onDelete?: () => void;
  onToggleSign?: () => void;
  onCursorLeft?: () => void;
  onCursorRight?: () => void;
};

export function TouchNumpad({
  onDigit,
  onDecimal,
  onDelete,
  onToggleSign,
  onCursorLeft,
  onCursorRight,
}: TouchNumpadInputProps = {}) {
  return (
    <div className="grid h-full w-max grid-cols-4 gap-4">
      {/* Row 1: 7 8 9 DEL */}
      <TouchNumpadButton onClick={() => onDigit?.("7")}>7</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("8")}>8</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("9")}>9</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDelete?.()}>
        <Icon name="lu:Delete" />
      </TouchNumpadButton>

      {/* Row 2: 4 5 6 <- */}
      <TouchNumpadButton onClick={() => onDigit?.("4")}>4</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("5")}>5</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("6")}>6</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onCursorLeft?.()}>
        <Icon name="lu:ArrowLeft" />
      </TouchNumpadButton>

      {/* Row 3: 1 2 3 -> */}
      <TouchNumpadButton onClick={() => onDigit?.("1")}>1</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("2")}>2</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDigit?.("3")}>3</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onCursorRight?.()}>
        <Icon name="lu:ArrowRight" />
      </TouchNumpadButton>

      {/* Row 4: 0 (spans 2 cols) · +- */}
      <TouchNumpadButton onClick={() => onDigit?.("0")} className="col-span-2">
        0
      </TouchNumpadButton>
      <TouchNumpadButton onClick={() => onDecimal?.()}>•</TouchNumpadButton>
      <TouchNumpadButton onClick={() => onToggleSign?.()}>
        <Icon name="lu:Diff" />
      </TouchNumpadButton>
    </div>
  );
}

type TouchNumpadButtonProps = {
  children: React.ReactNode;
  onClick: () => void;
  className?: string;
};

function TouchNumpadButton({
  children,
  onClick,
  className,
}: TouchNumpadButtonProps) {
  return (
    <Button
      className={`h-full font-mono text-2xl font-normal ${className?.includes("col-span-2") ? "w-full" : "w-22"} ${className || ""}`}
      variant="outline"
      onMouseDown={(e) => {
        // Prevent the button from stealing focus from the input
        e.preventDefault();
      }}
      onClick={() => {
        // Don't prevent default here - just call the onClick handler
        onClick();
      }}
    >
      {children}
    </Button>
  );
}
