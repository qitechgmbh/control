import { Preset } from "@/lib/preset/preset";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React, { JSX, useState } from "react";
import { TouchButton } from "../touch/TouchButton";
import { DialogHeader } from "../ui/dialog";
import { Separator } from "@radix-ui/react-select";
import { Icon } from "../Icon";

export type PresetShowDialogProps<T> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  renderPreview: (preset: Preset<T>) => JSX.Element;
};

export function PresetShowDialog<T>({
  preset,
  onApply,
  renderPreview,
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
          <DialogTitle>
            Preset <i>{preset.name}</i>
          </DialogTitle>
          <DialogDescription>
            Applying presets carelessly might damage machines.
          </DialogDescription>
        </DialogHeader>
        <Separator />

        <div className="text-sm text-gray-500">
          Last modification: {preset.lastModified.toISOString() || "N/A"}
          <br />
          <br />
          {renderPreview(preset)}
        </div>

        <Separator />

        <TouchButton
          className="flex-shrink-0"
          variant="outline"
          onClick={() => handleApply(preset)}
          icon="lu:HardDriveDownload"
        >
          Apply to Machine
        </TouchButton>
      </DialogContent>
    </Dialog>
  );
}
