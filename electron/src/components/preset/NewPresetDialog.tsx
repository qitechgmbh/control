import { PresetData, PresetSchema } from "@/lib/preset/preset";
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
import { PresetPreviewEntries, PresetPreviewTable } from "./PresetPreviewTable";
import { Input } from "../ui/input";

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

  const handleSave = () => {
    setOpen(false);
    setName("");
    onSave(name);
  };

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

        <Input
          placeholder="New Preset Name"
          onChange={(e) => setName(e.target.value)}
          className="w-full"
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
