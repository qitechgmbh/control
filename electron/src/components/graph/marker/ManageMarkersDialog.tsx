import React from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";
import type { Marker } from "@/stores/markerStore";

type ManageMarkersDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  markers: Marker[];
  onRemoveMarker: (marker: Marker) => void;
  onClearMarkers: () => void;
};

function formatMarkerTimestamp(timestamp: number) {
  try {
    return new Date(timestamp).toLocaleString();
  } catch {
    return `${timestamp}`;
  }
}

export function ManageMarkersDialog({
  open,
  onOpenChange,
  markers,
  onRemoveMarker,
  onClearMarkers,
}: ManageMarkersDialogProps) {
  const sortedMarkers = [...markers].sort((a, b) => b.timestamp - a.timestamp);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Icon name="lu:BookmarkMinus" />
            Remove Markers
          </DialogTitle>
          <DialogDescription>
            Remove individual markers or clear all markers for this machine.
          </DialogDescription>
        </DialogHeader>
        <Separator />

        <div className="max-h-[60vh] overflow-y-auto">
          {sortedMarkers.length === 0 ? (
            <div className="rounded-xl border border-dashed border-gray-300 bg-gray-50 px-4 py-8 text-center text-sm text-gray-500">
              No saved markers.
            </div>
          ) : (
            <div className="flex flex-col gap-3">
              {sortedMarkers.map((marker) => (
                <div
                  key={`${marker.timestamp}-${marker.name}`}
                  className="flex items-center justify-between gap-4 rounded-xl border border-gray-200 px-4 py-3"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span
                        className="h-3 w-3 shrink-0 rounded-full border border-gray-300"
                        style={{ backgroundColor: marker.color || "#000000" }}
                      />
                      <span className="truncate font-semibold text-gray-900">
                        {marker.name}
                      </span>
                    </div>
                    <div className="mt-1 text-sm text-gray-500">
                      {formatMarkerTimestamp(marker.timestamp)}
                    </div>
                  </div>
                  <TouchButton
                    variant="outline"
                    icon="lu:Trash2"
                    className="h-auto border-red-200 bg-red-50 px-3 py-2 text-red-700 hover:bg-red-100"
                    onClick={() => onRemoveMarker(marker)}
                  >
                    Remove
                  </TouchButton>
                </div>
              ))}
            </div>
          )}
        </div>

        <Separator />
        <div className="flex gap-4">
          <TouchButton
            variant="outline"
            icon="lu:X"
            className="h-21 flex-1"
            onClick={() => onOpenChange(false)}
          >
            Close
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:Trash2"
            className="h-21 flex-1 border-red-200 bg-red-50 text-red-700 hover:bg-red-100"
            onClick={onClearMarkers}
            disabled={sortedMarkers.length === 0}
          >
            Clear All
          </TouchButton>
        </div>
      </DialogContent>
    </Dialog>
  );
}
