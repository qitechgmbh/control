import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/icons/Icon";
import { SaveFileResult } from "@/helpers/file_export_helpers";

export type ExportResultDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  result: SaveFileResult | null;
  isEjectLoading: boolean;
  onEject: () => void;
};

/**
 * Shared post-export popup: shows where a file was saved (or the error),
 * and — when the export landed on removable media — an "Eject USB Drive"
 * action alongside Close, so the user doesn't pull the drive before writes
 * are flushed.
 */
export function ExportResultDialog({
  open,
  onOpenChange,
  result,
  isEjectLoading,
  onEject,
}: ExportResultDialogProps) {
  if (!result) return null;

  // Files that did land, also when another format of the same export failed.
  const saved =
    result.filePaths ??
    (result.success && result.filePath ? [result.filePath] : []);
  const title = result.success
    ? "Export Complete"
    : saved.length > 0
      ? "Export Partly Failed"
      : "Export Failed";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex flex-row items-center gap-2">
            <Icon
              name={result.success ? "lu:CircleCheck" : "lu:CircleX"}
              className={result.success ? "text-green-600" : "text-red-600"}
            />
            {title}
          </DialogTitle>
          <DialogDescription className="whitespace-pre-line">
            {saved.length === 0
              ? result.error
              : saved.length === 1
                ? `Saved to ${saved[0]}`
                : "Saved:"}
          </DialogDescription>
        </DialogHeader>
        {saved.length > 1 && (
          <ul className="list-disc pl-5 text-sm break-all text-gray-700">
            {saved.map((path) => (
              <li key={path}>{path}</li>
            ))}
          </ul>
        )}
        {saved.length > 0 && !result.success && (
          <p className="text-sm whitespace-pre-line text-red-600">
            {result.error}
          </p>
        )}
        <DialogFooter>
          {saved.length > 0 && result.isRemovable && (
            <TouchButton
              variant="outline"
              icon="lu:Usb"
              isLoading={isEjectLoading}
              onClick={onEject}
            >
              Eject USB Drive
            </TouchButton>
          )}
          <TouchButton variant="outline" onClick={() => onOpenChange(false)}>
            Close
          </TouchButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
