import React from "react";
import { Preset } from "@/lib/preset/preset";
import { TouchButton } from "@/components/touch/TouchButton";
import { PresetShowDialog } from "./PresetShowDialog";
import { Icon } from "../Icon";
import { PresetPreviewEntry } from "./PresetPreviewTable";

export type PresetCardProps<T> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  onOverwrite: (preset: Preset<T>) => void;
  onDelete: (preset: Preset<T>) => void;
  previewEntries: PresetPreviewEntry<T>[];
};

export function PresetCard<T>({
  preset,
  onApply,
  onOverwrite,
  onDelete,
  previewEntries,
}: PresetCardProps<T>) {
  return (
    <div className="flex flex-row items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div>
          <div className="flex flex-row gap-2 truncate text-lg font-semibold">
            <Icon name="lu:Check" className="text-green-500" />
            {preset.name}
          </div>
          <div className="text-sm text-gray-500">
            {preset.lastModified?.toLocaleString() || "Unknown date"}
          </div>
          <div className="text-green-500">This preset is active</div>
        </div>
      </div>
      <div className="flex gap-2">
        <TouchButton
          className="flex-shrink-0"
          variant="outline"
          onClick={() => onOverwrite(preset)}
        >
          <Icon name="lu:HardDriveUpload" />
          Overwrite
        </TouchButton>
        <PresetShowDialog
          preset={preset}
          onApply={onApply}
          previewEntries={previewEntries}
        ></PresetShowDialog>
        <TouchButton
          className="flex-shrink-0"
          variant="destructive"
          onClick={() => onDelete(preset)}
        >
          Delete
        </TouchButton>
      </div>
    </div>
  );
}
