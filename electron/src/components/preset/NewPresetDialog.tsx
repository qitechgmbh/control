import { PresetData, PresetSchema } from "@/lib/preset/preset";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { TouchButton } from "../touch/TouchButton";
import { DialogHeader } from "../ui/dialog";
import { Icon } from "../Icon";
import { Separator } from "../ui/separator";
import { PresetPreviewEntries, PresetPreviewTable } from "./PresetPreviewTable";
import { Input } from "../ui/input";
import { TouchNumpad } from "@/components/touch/TouchNumpad";

export type NewPresetDialogProps<T extends PresetSchema> = {
  currentState?: PresetData<T>;
  previewEntries: PresetPreviewEntries<T>;
  onSave: (name: string) => void;
};

export function NewPresetDialog<T extends PresetSchema>({
  currentState,
  onSave,
  previewEntries,
}: NewPresetDialogProps<T>) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const nameInputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const numpadRef = useRef<HTMLDivElement>(null);
  const [numpadOpen, setNumpadOpen] = useState(false);
  const [numpadPosition, setNumpadPosition] = useState({ left: 0, top: 0 });
  const nameContainerRef = useRef<HTMLDivElement>(null);

  const handleSave = () => {
    setOpen(false);
    setName("");
    onSave(name);
  };

  // Position numpad once when it opens
  useEffect(() => {
    if (!numpadOpen || !dialogRef.current) return;
    const rect = dialogRef.current.getBoundingClientRect();
    setNumpadPosition({
      left: rect.right + 20,
      top: rect.top + rect.height / 2,
    });
  }, [numpadOpen]);

  // Close numpad when clicking outside input/numpad
  useEffect(() => {
    if (!numpadOpen) return;

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      const insideName = nameContainerRef.current?.contains(target);
      const insideNumpad = numpadRef.current?.contains(target);
      if (!insideName && !insideNumpad) {
        setNumpadOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  }, [numpadOpen]);

  // Keep focus on input field when numpad is opened
  useEffect(() => {
    if (numpadOpen && nameInputRef.current) {
      // Use setTimeout to ensure this runs after any other focus changes
      setTimeout(() => {
        if (nameInputRef.current) {
          nameInputRef.current.focus();
        }
      }, 0);
    }
  }, [numpadOpen]);

  // Numpad handlers for name input
  const numpadHandlers = useMemo(() => {
    const ensureFocus = () => {
      if (
        nameInputRef.current &&
        document.activeElement !== nameInputRef.current
      ) {
        nameInputRef.current.focus();
      }
    };

    const updateCursorPosition = (position: number) => {
      setTimeout(() => {
        if (nameInputRef.current) {
          nameInputRef.current.setSelectionRange(position, position);
        }
      }, 0);
    };

    const getCurrentValue = () => {
      return name || "";
    };

    return {
      appendDigit: (digit: string) => {
        if (!nameInputRef.current) return;

        ensureFocus();
        const input = nameInputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const currentValue = getCurrentValue();
        const newValue =
          currentValue.slice(0, start) + digit + currentValue.slice(end);

        setName(newValue);
        updateCursorPosition(start + 1);
      },

      deleteChar: () => {
        if (!nameInputRef.current) return;

        ensureFocus();
        const input = nameInputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const currentValue = getCurrentValue();

        let newValue: string;
        let newPosition: number;

        if (start !== end) {
          // Delete selection
          newValue = currentValue.slice(0, start) + currentValue.slice(end);
          newPosition = start;
        } else if (start > 0) {
          // Backspace
          newValue =
            currentValue.slice(0, start - 1) + currentValue.slice(start);
          newPosition = start - 1;
        } else {
          return;
        }

        setName(newValue);
        updateCursorPosition(newPosition);
      },

      moveCursorLeft: () => {
        if (!nameInputRef.current) return;

        ensureFocus();
        const currentPos = nameInputRef.current.selectionStart || 0;
        if (currentPos > 0) {
          nameInputRef.current.setSelectionRange(
            currentPos - 1,
            currentPos - 1,
          );
        }
      },

      moveCursorRight: () => {
        if (!nameInputRef.current) return;

        ensureFocus();
        const currentPos = nameInputRef.current.selectionStart || 0;
        const currentValue = getCurrentValue();
        if (currentPos < currentValue.length) {
          nameInputRef.current.setSelectionRange(
            currentPos + 1,
            currentPos + 1,
          );
        }
      },
    };
  }, [name]);

  return (
    <>
      <Dialog open={open} onOpenChange={setOpen} modal>
        <DialogTrigger asChild>
          <TouchButton disabled={!currentState} icon="lu:SquarePlus">
            Create New Preset
          </TouchButton>
        </DialogTrigger>

        <DialogContent
          ref={dialogRef}
          onInteractOutside={(e) => e.preventDefault()}
          onPointerDownOutside={(e) => e.preventDefault()}
          onEscapeKeyDown={(e) => e.preventDefault()}
        >
          <DialogHeader>
            <DialogTitle className="flex flex-row items-center gap-2">
              <Icon name="lu:SquarePlus" />
              Create a New Preset
            </DialogTitle>
            <DialogDescription>
              Save the configuration of this machine to use it later.
            </DialogDescription>
          </DialogHeader>
          <Separator />

          <div ref={nameContainerRef} className="flex items-center gap-2">
            <Input
              ref={nameInputRef}
              placeholder="New Preset Name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onFocus={() => setNumpadOpen(true)}
              onClick={() => setNumpadOpen(true)}
              onBlur={(event) => {
                const next = event.relatedTarget as Node | null;
                if (
                  nameContainerRef.current?.contains(next) ||
                  numpadRef.current?.contains(next)
                ) {
                  return;
                }
                setNumpadOpen(false);
              }}
              className="w-full"
            />
          </div>
        <div className="flex flex-col gap-6 text-sm">
          <span>Current Settings:</span>
          <PresetPreviewTable entries={previewEntries} data={currentState} />
        </div>

        <Separator />
          <div className="flex flex-row gap-4">
            <TouchButton
              variant="outline"
              icon="lu:X"
              className="h-21 flex-1"
              onClick={() => setOpen(false)}
            >
              Abort
            </TouchButton>

            <TouchButton
              className="h-21 flex-1 flex-shrink-0"
              onClick={handleSave}
              icon="lu:Save"
              disabled={!name}
            >
              Save
            </TouchButton>
          </div>
        </DialogContent>
      </Dialog>
      {/* Numpad as separate window right of dialog */}
      {numpadOpen && (
        <div
          ref={numpadRef}
          data-numpad
          className="fixed z-[100] w-auto rounded-md border border-neutral-200 bg-white p-4 shadow-md dark:border-neutral-800 dark:bg-neutral-950"
          style={{
            left: `${numpadPosition.left}px`,
            top: `${numpadPosition.top}px`,
            transform: "translateY(-50%)",
            pointerEvents: "auto",
          }}
          tabIndex={-1}
          onMouseDown={(e) => {
            // Prevent clicks on numpad from closing the dialog and stealing focus from input
            e.preventDefault();
            e.stopPropagation();
            // Ensure input field keeps focus
            if (nameInputRef.current) {
              nameInputRef.current.focus();
            }
          }}
          onClick={(e) => {
            // Prevent clicks on numpad from closing the dialog
            e.stopPropagation();
            // Ensure input field keeps focus
            if (nameInputRef.current) {
              nameInputRef.current.focus();
            }
          }}
          onKeyDown={(e) => {
            // Prevent Escape or other keys from bubbling and closing the dialog
            e.stopPropagation();
          }}
        >
          <TouchNumpad
            onDigit={numpadHandlers.appendDigit}
            onDelete={numpadHandlers.deleteChar}
            onCursorLeft={numpadHandlers.moveCursorLeft}
            onCursorRight={numpadHandlers.moveCursorRight}
          />
        </div>
      )}
    </>
  );
}
