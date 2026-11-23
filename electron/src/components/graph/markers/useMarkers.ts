import { useCallback, useEffect, useRef } from "react";
import { toast } from "sonner";
import {
  machineKey,
  subscribeMarkers,
  useMarkerStore,
  type Marker,
} from "@/stores/markerStore";
import { useGraphHistoryMachine } from "../history/GraphHistoryContext";

// Stable empty array to avoid "getSnapshot should be cached" infinite loop
const EMPTY_MARKERS: Marker[] = [];

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

/**
 * Markers of the machine given by the surrounding `GraphHistoryProvider`, loaded from
 * the database and kept fresh while mounted. Without a provider there are no markers
 * and `enabled` is false.
 */
export function useMarkers() {
  const machine = useGraphHistoryMachine();
  const key = machine ? machineKey(machine) : null;

  // The identification object may be re-created every render; `key` identifies it.
  const machineRef = useRef(machine);
  useEffect(() => {
    machineRef.current = machine;
  });

  const markers = useMarkerStore((state) =>
    key ? (state.markersByMachine[key] ?? EMPTY_MARKERS) : EMPTY_MARKERS,
  );
  const addAction = useMarkerStore((state) => state.addMarker);
  const removeAction = useMarkerStore((state) => state.removeMarker);
  const clearAction = useMarkerStore((state) => state.clearMarkers);

  useEffect(() => {
    const current = machineRef.current;
    if (!key || !current) return;
    return subscribeMarkers(current);
  }, [key]);

  const addMarker = useCallback(
    async (name: string, timestamp: number, color?: string) => {
      const current = machineRef.current;
      if (!current) return;
      try {
        await addAction(current, { name, timestamp, color });
      } catch (error) {
        toast.error(`Failed to add marker: ${errorText(error)}`);
      }
    },
    [addAction],
  );

  const removeMarker = useCallback(
    async (id: string) => {
      const current = machineRef.current;
      if (!current) return;
      try {
        await removeAction(current, id);
      } catch (error) {
        toast.error(`Failed to remove marker: ${errorText(error)}`);
      }
    },
    [removeAction],
  );

  const clearMarkers = useCallback(async () => {
    const current = machineRef.current;
    if (!current) return;
    try {
      await clearAction(current);
    } catch (error) {
      toast.error(`Failed to clear markers: ${errorText(error)}`);
    }
  }, [clearAction]);

  return {
    enabled: key !== null,
    markers,
    addMarker,
    removeMarker,
    clearMarkers,
  };
}
