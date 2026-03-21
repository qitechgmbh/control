import React, { useState, useEffect, useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import { TimeInput } from "@/components/time/TimeInput";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { TouchKeyboard } from "@/components/touch/TouchKeyboard";

type AddMarkerDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAddMarker: (name: string, timestamp: number, color?: string) => void;
  currentTimestamp: number | null;
  defaultName?: string;
  // Existing marker names to prevent duplicates (case-insensitive)
  existingNames?: string[];
};

export function AddMarkerDialog({
  open,
  onOpenChange,
  onAddMarker,
  currentTimestamp,
  defaultName = "",
  existingNames = [],
}: AddMarkerDialogProps) {
  const [name, setName] = useState(defaultName);
  const [keyboardVisible, setKeyboardVisible] = useState(false);
  const [selectedTimestamp, setSelectedTimestamp] = useState<number | null>(
    null,
  );
  const [color, setColor] = useState("#000000");
  const inputRef = useRef<HTMLInputElement>(null);
  // Shown when user tries to add a marker whose name already exists
  const [duplicateNameError, setDuplicateNameError] = useState(false);

  // Reset form only when dialog opens or closes; do not reset when currentTimestamp
  // updates while open (e.g. from graph) or we overwrite the user's time input
  useEffect(() => {
    if (open) {
      setName(defaultName);
      setKeyboardVisible(true);
      // Keep timestamp unset until the user explicitly chooses a time.
      // handleAdd falls back to currentTimestamp / Date.now() on submit.
      setSelectedTimestamp(null);
      setDuplicateNameError(false);
    } else {
      setName("");
      setKeyboardVisible(false);
      setSelectedTimestamp(null);
      setDuplicateNameError(false);
    }
  }, [open]);

  const showKeyboard = () => {
    setKeyboardVisible(true);
    inputRef.current?.focus();
  };

  const handleAdd = () => {
    if (!name.trim()) return;

    // Reject duplicate names (compare trimmed, case-insensitive)
    const trimmedName = name.trim();
    const isDuplicate = existingNames.some(
      (existing) => existing.trim().toLowerCase() === trimmedName.toLowerCase(),
    );
    if (isDuplicate) {
      setDuplicateNameError(true);
      return;
    }

    setDuplicateNameError(false);

    // Use selected time if set, else context/graph time, else now
    const timestamp = selectedTimestamp ?? currentTimestamp ?? Date.now();
    if (!timestamp) return;

    onAddMarker(trimmedName, timestamp, color);
    onOpenChange(false);
  };

  const handleCancel = () => {
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-md"
        onOpenAutoFocus={(event) => {
          event.preventDefault();
          showKeyboard();
        }}
      >
        <DialogHeader>
          <DialogTitle className="flex flex-row items-center gap-2">
            <Icon name="lu:Bookmark" />
            Add Marker
          </DialogTitle>
          <DialogDescription>
            Create a marker for all graphs of this machine at the current time.
          </DialogDescription>
        </DialogHeader>
        <Separator />

        <div className="flex flex-col gap-4">
          {/* Name Input */}
          <div className="flex flex-col gap-2">
            <Label htmlFor="marker-name">Marker Name</Label>
            <div className="flex gap-3">
              <Input
                id="marker-name"
                ref={inputRef}
                readOnly
                value={name}
                placeholder="Tap to enter marker name"
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
            {duplicateNameError && (
              <p className="text-destructive text-sm">
                A marker with this name already exists.
              </p>
            )}
          </div>

          {/* Time Input (optional) */}
          <div className="flex flex-col gap-2">
            <Label>Time (optional)</Label>
            <TimeInput
              timestamp={selectedTimestamp}
              onTimeChange={setSelectedTimestamp}
              onClear={() => setSelectedTimestamp(null)}
            />
            <p className="text-muted-foreground text-xs">
              Leave empty to use current time
            </p>
          </div>

          {/* Color Input */}
          <div className="flex flex-col gap-2">
            <Label htmlFor="marker-color">Color</Label>
            <div className="flex items-center gap-3">
              <input
                id="marker-color"
                type="color"
                value={color}
                onChange={(e) => setColor(e.target.value)}
                className="border-input h-9 w-20 cursor-pointer rounded-md border"
              />
              <Input
                type="text"
                value={color}
                onChange={(e) => setColor(e.target.value)}
                placeholder="#000000"
              />
            </div>
          </div>
        </div>

        <TouchKeyboard
          value={name}
          onChange={(value) => {
            setName(value);
            setDuplicateNameError(false);
          }}
          onEnter={handleAdd}
          visible={keyboardVisible}
        />

        <Separator />
        <div className="flex flex-row gap-4">
          <TouchButton
            variant="outline"
            icon="lu:X"
            className="h-21 flex-1"
            onClick={handleCancel}
          >
            Abort
          </TouchButton>
          <TouchButton
            className="h-21 flex-1 flex-shrink-0"
            onClick={handleAdd}
            icon="lu:Bookmark"
            disabled={!name.trim()}
          >
            Add Marker
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
