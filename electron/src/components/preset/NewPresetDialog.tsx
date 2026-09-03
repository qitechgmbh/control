import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React, { useRef, useState } from "react";
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
  const [keyboardVisible, setKeyboardVisible] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const resetDialog = () => {
    setOpen(false);
    setName("");
    setKeyboardVisible(false);
  };

  const handleOpenChange = (nextOpen: boolean) => {
    setOpen(nextOpen);

    if (!nextOpen) {
      setName("");
      setKeyboardVisible(false);
      return;
    }

    setKeyboardVisible(true);
  };

  const handleSave = () => {
    const trimmedName = name.trim();
    if (!trimmedName) {
      return;
    }

    onSave(trimmedName);
    resetDialog();
  };

  const showKeyboard = () => {
    setKeyboardVisible(true);
    inputRef.current?.focus();
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <TouchButton disabled={!currentState} icon="lu:SquarePlus">
          Create New Preset
        </TouchButton>
      </DialogTrigger>

      <DialogContent
        className="max-h-[90vh] overflow-y-auto sm:max-w-4xl"
        onOpenAutoFocus={(event) => {
          event.preventDefault();
          showKeyboard();
        }}
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

        <div className="flex flex-col gap-3">
          <label
            htmlFor="new-preset-name"
            className="text-sm font-medium tracking-wide"
          >
            Preset Name
          </label>
          <div className="flex gap-3">
            <Input
              id="new-preset-name"
              ref={inputRef}
              readOnly
              value={name}
              placeholder="Tap to enter preset name"
              onFocus={showKeyboard}
              onClick={showKeyboard}
              className="h-14 text-lg"
            />
            <TouchButton
              type="button"
              variant="outline"
              icon={keyboardVisible ? "lu:KeyboardOff" : "lu:Keyboard"}
              onClick={() => {
                if (keyboardVisible) {
                  setKeyboardVisible(false);
                  return;
                }

                showKeyboard();
              }}
            >
              {keyboardVisible ? "Hide" : "Edit"}
            </TouchButton>
          </div>
        </div>

        <TouchKeyboard
          value={name}
          onChange={setName}
          onEnter={handleSave}
          visible={keyboardVisible}
        />

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
            onClick={resetDialog}
          >
            Abort
          </TouchButton>

          <TouchButton
            className="h-21 flex-1 flex-shrink-0"
            onClick={handleSave}
            icon="lu:Save"
            disabled={!name.trim()}
          >
            Save
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
