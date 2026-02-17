import { useCallback } from "react";
import { useMarkerStore, type Marker } from "@/stores/markerStore";

/**
 * Hook for marker management backed by a centralized Zustand store.
 * Single source of truth ensures all markers persist correctly across navigation
 * and multiple graph instances.
 */
export function useMarkerManager(machineId: string) {
  const markers = useMarkerStore(
    (state) => state.markersByMachine[machineId] ?? [],
  );
  const addMarkerAction = useMarkerStore((state) => state.addMarker);
  const removeMarkerAction = useMarkerStore((state) => state.removeMarker);
  const clearMarkersAction = useMarkerStore((state) => state.clearMarkers);

  const addMarker = useCallback(
    (name: string, timestamp: number, color?: string, value?: number) => {
      addMarkerAction(machineId, { name, timestamp, color, value });
      return { name, timestamp, color, value } as Marker;
    },
    [machineId, addMarkerAction],
  );

  const removeMarker = useCallback(
    (timestamp: number) => {
      removeMarkerAction(machineId, timestamp);
    },
    [machineId, removeMarkerAction],
  );

  const clearMarkers = useCallback(() => {
    clearMarkersAction(machineId);
  }, [machineId, clearMarkersAction]);

  return {
    markers,
    addMarker,
    removeMarker,
    clearMarkers,
  };
}
