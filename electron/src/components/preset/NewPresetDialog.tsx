import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React, { useCallback, useEffect, useRef, useState } from "react";
import { TouchButton } from "../touch/TouchButton";
import { DialogHeader } from "../ui/dialog";
import { Icon } from "../Icon";
import { Separator } from "../ui/separator";
import { PresetPreviewEntries, PresetPreviewTable } from "./PresetPreviewTable";
import { Input } from "../ui/input";
import { TouchKeyboard } from "../touch/TouchKeyboard";

export type NewPresetDialogProps<T> = {
  currentState?: T;
  previewEntries: PresetPreviewEntries<T>;
  onSave: (name: string) => void;
};

export function NewPresetDialog<T>({
  currentState,
  onSave,
  previewEntries,
}: NewPresetDialogProps<T>) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [keyboardOpen, setKeyboardOpen] = useState(false);
  const [keyboardPosition, setKeyboardPosition] = useState({ left: 0, top: 0 });
  const inputRef = useRef<HTMLInputElement>(null);
  const inputContainerRef = useRef<HTMLDivElement>(null);
  const keyboardRef = useRef<HTMLDivElement>(null);

  const handleSave = () => {
    setOpen(false);
    setKeyboardOpen(false);
    setName("");
    onSave(name);
  };

  const ensureInputFocus = () => {
    if (inputRef.current && document.activeElement !== inputRef.current) {
      inputRef.current.focus();
    }
  };

  const updateCursorPosition = (position: number) => {
    setTimeout(() => {
      if (inputRef.current) {
        inputRef.current.setSelectionRange(position, position);
      }
    }, 0);
  };

  const getSelection = () => {
    const start = inputRef.current?.selectionStart ?? name.length;
    const end = inputRef.current?.selectionEnd ?? name.length;
    return { start, end };
  };

  const insertText = (value: string) => {
    ensureInputFocus();
    const { start, end } = getSelection();
    const newValue = name.slice(0, start) + value + name.slice(end);
    setName(newValue);
    updateCursorPosition(start + value.length);
  };

  const deleteCharacter = () => {
    ensureInputFocus();
    const { start, end } = getSelection();
    if (start !== end) {
      setName(name.slice(0, start) + name.slice(end));
      updateCursorPosition(start);
      return;
    }
    if (start === 0) return;
    setName(name.slice(0, start - 1) + name.slice(end));
    updateCursorPosition(start - 1);
  };

  const clearName = () => {
    setName("");
    updateCursorPosition(0);
  };

  const moveCursorLeft = () => {
    ensureInputFocus();
    const pos = inputRef.current?.selectionStart ?? 0;
    updateCursorPosition(Math.max(0, pos - 1));
  };

  const moveCursorRight = () => {
    ensureInputFocus();
    const pos = inputRef.current?.selectionStart ?? 0;
    updateCursorPosition(Math.min(name.length, pos + 1));
  };

  const updateKeyboardPosition = useCallback(() => {
    if (!keyboardOpen || !inputContainerRef.current) return;
    const rect = inputContainerRef.current.getBoundingClientRect();
    setKeyboardPosition({
      left: rect.right + 20,
      top: rect.top + rect.height / 2,
    });
  }, [keyboardOpen]);

  useEffect(() => {
    if (!open) {
      setKeyboardOpen(false);
    }
  }, [open]);

  useEffect(() => {
    updateKeyboardPosition();
  }, [keyboardOpen, updateKeyboardPosition]);

  useEffect(() => {
    window.addEventListener("resize", updateKeyboardPosition);
    return () => {
      window.removeEventListener("resize", updateKeyboardPosition);
    };
  }, [updateKeyboardPosition]);

  useEffect(() => {
    if (!keyboardOpen) return;
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      const insideInput = inputContainerRef.current?.contains(target);
      const insideKeyboard = keyboardRef.current?.contains(target);
      if (!insideInput && !insideKeyboard) {
        setKeyboardOpen(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  }, [keyboardOpen]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <TouchButton disabled={!currentState} icon="lu:SquarePlus">
          Create New Preset
        </TouchButton>
      </DialogTrigger>

      <DialogContent>
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

        <div ref={inputContainerRef}>
          <Input
            ref={inputRef}
            placeholder="New Preset Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onFocus={() => setKeyboardOpen(true)}
            className="w-full"
            autoFocus
          />
        </div>

        {keyboardOpen && open && (
          <div
            ref={keyboardRef}
            className="bg-background fixed z-[60] mx-8 flex w-min flex-col gap-6 rounded-2xl border p-6 shadow-2xl"
            style={{
              left: `${keyboardPosition.left}px`,
              top: `${keyboardPosition.top}px`,
              transform: "translateY(-50%)",
            }}
          >
            <TouchKeyboard
              onKey={insertText}
              onSpace={() => insertText(" ")}
              onDelete={deleteCharacter}
              onClear={clearName}
              onCursorLeft={moveCursorLeft}
              onCursorRight={moveCursorRight}
            />
          </div>
        )}

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
  );
}
