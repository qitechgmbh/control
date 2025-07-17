import React from "react";
import { Preset, PresetSchema } from "@/lib/preset/preset";
import { TouchButton } from "@/components/touch/TouchButton";
import { PresetShowDialog } from "./PresetShowDialog";
import { Icon } from "../Icon";
import { PresetPreviewEntry } from "./PresetPreviewTable";

export type PresetCardProps<T extends PresetSchema> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  onOverwrite: (preset: Preset<T>) => void;
  onDelete: (preset: Preset<T>) => void;
  previewEntries: PresetPreviewEntry<T>[];
  isReadOnly?: boolean;
  hideDate?: boolean;
  isActive?: boolean;
};

export function PresetCard<T extends PresetSchema>({
  preset,
  onApply,
  onOverwrite,
  onDelete,
  previewEntries,
  isReadOnly,
  hideDate,
  isActive,
}: PresetCardProps<T>) {
  return (
    <div className="flex flex-row items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div>
          <div className="flex flex-row gap-2 truncate text-lg font-semibold">
            {isActive && <Icon name="lu:Check" className="text-green-500" />}
            {preset.name}
          </div>
          {!hideDate && (
            <div className="text-sm text-gray-500">
              {preset.lastModified?.toLocaleString() || "Unknown date"}
            </div>
          )}
          {isActive && (
            <span className="text-green-500">This preset is active</span>
          )}
        </div>
      </div>
      <div className="flex gap-2">
        {!isReadOnly && (
          <TouchButton
            className="flex-shrink-0"
            variant="outline"
            onClick={() => onOverwrite(preset)}
          >
            <Icon name="lu:HardDriveUpload" />
            Overwrite
          </TouchButton>
        )}
        <PresetShowDialog
          preset={preset}
          onApply={onApply}
          previewEntries={previewEntries}
          hideDate={hideDate}
        ></PresetShowDialog>
        {!isReadOnly && (
          <TouchButton
            className="flex-shrink-0"
            variant="destructive"
            onClick={() => onDelete(preset)}
          >
            Delete
          </TouchButton>
        )}
      </div>
    </div>
  );
}
