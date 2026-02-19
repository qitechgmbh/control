import React, { useState, useEffect } from "react";
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
  const [selectedTimestamp, setSelectedTimestamp] = useState<number | null>(
    null,
  );
  const [color, setColor] = useState("#000000");
  // Shown when user tries to add a marker whose name already exists
  const [duplicateNameError, setDuplicateNameError] = useState(false);

  // Reset form only when dialog opens or closes; do not reset when currentTimestamp
  // updates while open (e.g. from graph) or we overwrite the user's time input
  useEffect(() => {
    if (open) {
      setName(defaultName);
      setSelectedTimestamp(currentTimestamp ?? Date.now());
      setDuplicateNameError(false);
    } else {
      setName("");
      setSelectedTimestamp(null);
      setDuplicateNameError(false);
    }
  }, [open]);

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
      <DialogContent className="sm:max-w-md">
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
            <Input
              id="marker-name"
              type="text"
              value={name}
              onChange={(e) => {
                setName(e.target.value);
                setDuplicateNameError(false);
              }}
              placeholder="Enter marker name"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleAdd();
                }
              }}
            />
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
              onClear={() =>
                setSelectedTimestamp(currentTimestamp ?? Date.now())
              }
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
