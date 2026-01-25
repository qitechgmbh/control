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
};

export function AddMarkerDialog({
  open,
  onOpenChange,
  onAddMarker,
  currentTimestamp,
  defaultName = "",
}: AddMarkerDialogProps) {
  const [name, setName] = useState(defaultName);
  const [selectedTimestamp, setSelectedTimestamp] = useState<number | null>(
    null,
  );
  const [color, setColor] = useState("#000000");

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (open) {
      setName(defaultName);
      // Always use current time when dialog opens
      setSelectedTimestamp(currentTimestamp);
    } else {
      setName("");
      setSelectedTimestamp(null);
    }
  }, [open, currentTimestamp, defaultName]);

  const handleAdd = () => {
    if (!name.trim()) return;

    // Always use current timestamp when dialog is opened (as per requirement)
    // The time input is optional and only for historical markers
    const timestamp = currentTimestamp || Date.now();
    if (!timestamp) return;

    onAddMarker(name.trim(), timestamp, color);
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
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter marker name"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleAdd();
                }
              }}
            />
          </div>

          {/* Time Input (optional) */}
          <div className="flex flex-col gap-2">
            <Label>Time (optional)</Label>
            <TimeInput
              timestamp={selectedTimestamp}
              onTimeChange={setSelectedTimestamp}
              onClear={() => setSelectedTimestamp(currentTimestamp)}
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
            disabled={!name.trim() || !(selectedTimestamp || currentTimestamp)}
          >
            Add Marker
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
