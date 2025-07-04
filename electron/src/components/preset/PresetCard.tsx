import React, { JSX } from "react";
import { Preset } from "@/lib/preset/preset";
import { TouchButton } from "@/components/touch/TouchButton";
import { PresetShowDialog } from "./PresetShowDialog";
import { Icon } from "../Icon";

export type PresetCardProps<T> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  onOverwrite: (preset: Preset<T>) => void;
  onDelete: (preset: Preset<T>) => void;
  renderPreview: (preset: Preset<T>) => JSX.Element;
};

export function PresetCard<T>({
  preset,
  onApply,
  onOverwrite,
  onDelete,
  renderPreview,
}: PresetCardProps<T>) {
  return (
    <div className="flex flex-row items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div>
          <div className="truncate text-lg font-semibold text-gray-900">
            {preset.name}
          </div>
          <div className="text-sm text-gray-500">
            {preset.lastModified?.toDateString() || "Unknown date"}
          </div>
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
          renderPreview={renderPreview}
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
