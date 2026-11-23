import React, { useState } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { AddMarkerDialog } from "./AddMarkerDialog";
import { ManageMarkersDialog } from "./ManageMarkersDialog";
import { useMarkers } from "./useMarkers";

type MarkerControlsProps = {
  /** Time a new marker gets if the user picks none; null means "now" when added. */
  defaultTimestamp: number | null;
};

/**
 * "Add Marker" / "Remove Marker" buttons with their dialogs, for the machine of the
 * surrounding `GraphHistoryProvider`. Renders nothing without one.
 */
export function MarkerControls({ defaultTimestamp }: MarkerControlsProps) {
  const { enabled, markers, addMarker, removeMarker, clearMarkers } =
    useMarkers();
  const [addOpen, setAddOpen] = useState(false);
  const [manageOpen, setManageOpen] = useState(false);

  if (!enabled) return null;

  return (
    <>
      <TouchButton
        onClick={() => setAddOpen(true)}
        variant="outline"
        className="h-auto bg-blue-600 px-3 py-3 text-base font-medium text-white hover:bg-blue-700"
      >
        Add Marker
      </TouchButton>
      <TouchButton
        onClick={() => setManageOpen(true)}
        variant="outline"
        className="h-auto border-red-200 bg-red-50 px-3 py-3 text-base font-medium text-red-700 hover:bg-red-100"
      >
        Remove Marker
      </TouchButton>

      <AddMarkerDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        // Machine-wide by timestamp; each graph derives its own y value.
        onAddMarker={(name, timestamp, color) =>
          void addMarker(name, timestamp, color)
        }
        defaultTimestamp={defaultTimestamp}
        existingNames={markers.map((m) => m.name)}
      />
      <ManageMarkersDialog
        open={manageOpen}
        onOpenChange={setManageOpen}
        markers={markers}
        onRemoveMarker={(marker) => void removeMarker(marker.id)}
        onClearMarkers={() => void clearMarkers()}
      />
    </>
  );
}
