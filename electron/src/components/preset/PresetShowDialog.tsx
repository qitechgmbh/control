import { Preset, PresetData } from "@/lib/preset/preset";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React, { useState } from "react";
import { TouchButton } from "../touch/TouchButton";
import { DialogHeader } from "../ui/dialog";
import { Icon } from "../Icon";
import { Separator } from "../ui/separator";
import { PresetPreviewEntry, PresetPreviewTable } from "./PresetPreviewTable";

export type PresetShowDialogProps<T extends PresetData> = {
  preset: Preset<T>;
  previewEntries: PresetPreviewEntry<T>[];
  onApply: (preset: Preset<T>) => void;
  hideDate?: boolean;
};

export function PresetShowDialog<T extends PresetData>({
  preset,
  onApply,
  previewEntries,
  hideDate,
}: PresetShowDialogProps<T>) {
  const [open, setOpen] = useState(false);

  const handleApply = (preset: Preset<T>) => {
    setOpen(false);
    onApply(preset);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <TouchButton variant="outline" icon="lu:Eye" className="w-max">
          Show and Apply
        </TouchButton>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex flex-row items-center gap-2">
            <Icon name="lu:Save" />
            {preset.name}
          </DialogTitle>
          <DialogDescription>
            Applying presets carelessly might damage machines.
          </DialogDescription>
        </DialogHeader>
        <Separator />

        <div className="flex flex-col gap-6 text-sm text-gray-500">
          {!hideDate && (
            <div>
              Last modification: {preset.lastModified.toLocaleString() || "N/A"}
            </div>
          )}
          <PresetPreviewTable entries={previewEntries} preset={preset} />
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
            onClick={() => handleApply(preset)}
            icon="lu:HardDriveDownload"
          >
            Apply to Machine
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
