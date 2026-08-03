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
import { Icon } from "@/components/Icon";
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

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex flex-row items-center gap-2">
            <Icon
              name={result.success ? "lu:CircleCheck" : "lu:CircleX"}
              className={result.success ? "text-green-600" : "text-red-600"}
            />
            {result.success ? "Export Complete" : "Export Failed"}
          </DialogTitle>
          <DialogDescription>
            {result.success ? `Saved to ${result.filePath}` : result.error}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          {result.success && result.isRemovable && (
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
