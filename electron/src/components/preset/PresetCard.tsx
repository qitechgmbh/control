import React, { useState } from "react";
import { Preset, PresetSchema } from "@/lib/preset/preset";
import { TouchButton } from "@/components/touch/TouchButton";
import { PresetShowDialog } from "./PresetShowDialog";
import { Icon } from "../Icon";
import { PresetPreviewEntries } from "./PresetPreviewTable";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";

type PresetCardMenu = {
  onOverwrite?: () => void;
  onDelete?: () => void;
  onExport?: () => void;
};

export function PresetCardMenu({
  onOverwrite,
  onDelete,
  onExport,
}: PresetCardMenu) {
  const [open, setOpen] = useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <TouchButton variant="outline" icon="lu:Menu" />
      </PopoverTrigger>

      <PopoverContent className="flex flex-col gap-4">
        {onOverwrite && (
          <TouchButton
            className="flex-shrink-0"
            variant="outline"
            onClick={onOverwrite}
          >
            <Icon name="lu:HardDriveUpload" />
            Overwrite
          </TouchButton>
        )}
        {onExport && (
          <TouchButton
            className="flex-shrink-0"
            variant="outline"
            onClick={onExport}
          >
            <Icon name="lu:Download" />
            Export
          </TouchButton>
        )}
        {onDelete && (
          <TouchButton
            className="flex-shrink-0"
            variant="destructive"
            onClick={onDelete}
          >
            Delete
          </TouchButton>
        )}
      </PopoverContent>
    </Popover>
  );
}

export type PresetCardProps<T extends PresetSchema> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  onOverwrite: (preset: Preset<T>) => void;
  onDelete: (preset: Preset<T>) => void;
  onExport?: (preset: Preset<T>) => void;
  previewEntries: PresetPreviewEntries<T>;
  isReadOnly?: boolean;
  hideDate?: boolean;
  isActive?: boolean;
};

export function PresetCard<T extends PresetSchema>({
  preset,
  onApply,
  onOverwrite,
  onDelete,
  onExport,
  previewEntries,
  isReadOnly,
  hideDate,
  isActive,
}: PresetCardProps<T>) {
  return (
    <div className="flex flex-row items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div>
          <div className="flex flex-row gap-2 text-lg font-semibold">
            {isActive && <Icon name="lu:Check" className="text-green-500" />}
            <span title={preset.name} className="w-60 truncate">
              {preset.name}
            </span>
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
        <PresetShowDialog
          preset={preset}
          onApply={onApply}
          previewEntries={previewEntries}
          hideDate={hideDate}
        />
        {isReadOnly ? (
          <TouchButton variant="outline" icon="lu:Menu" disabled />
        ) : (
          <PresetCardMenu
            onOverwrite={() => onOverwrite(preset)}
            onDelete={() => onDelete(preset)}
            onExport={() => onExport && onExport(preset)}
          />
        )}
      </div>
    </div>
  );
}
