import { useState, useCallback } from "react";
import { toast } from "sonner";
import { ejectUsb, SaveFileResult } from "@/helpers/file_export_helpers";

/**
 * Shared state/logic behind the common export result popup
 * (ExportResultDialog): tracks the last save attempt and offers a "safely
 * eject" action when it landed on removable media.
 */
export function useExportDialog() {
  const [open, setOpen] = useState(false);
  const [result, setResult] = useState<SaveFileResult | null>(null);
  const [isEjectLoading, setIsEjectLoading] = useState(false);

  const notifyResult = useCallback((newResult: SaveFileResult) => {
    // Don't bother the user with a popup if they simply canceled the save
    // dialog — that's not a failure worth surfacing.
    if (!newResult.success && newResult.error === "Export cancelled by user") {
      return;
    }
    setResult(newResult);
    setOpen(true);
  }, []);

  const handleEject = useCallback(async () => {
    if (!result?.mountPath) return;
    setIsEjectLoading(true);
    try {
      const ejectResult = await ejectUsb(result.mountPath);
      if (ejectResult.success) {
        toast.success("USB drive safely ejected");
        setResult((prev) => (prev ? { ...prev, isRemovable: false } : prev));
      } else {
        toast.error(`Failed to eject USB drive: ${ejectResult.error}`);
      }
    } catch (error) {
      toast.error(`Failed to eject USB drive: ${error}`);
    } finally {
      setIsEjectLoading(false);
    }
  }, [result]);

  return {
    notifyResult,
    dialogProps: {
      open,
      onOpenChange: setOpen,
      result,
      isEjectLoading,
      onEject: handleEject,
    },
  };
}
