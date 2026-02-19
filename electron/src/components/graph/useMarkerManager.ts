import { useCallback } from "react";
import { useMarkerStore, type Marker } from "@/stores/markerStore";

// Stable empty array to avoid "getSnapshot should be cached" infinite loop
const EMPTY_MARKERS: Marker[] = [];


export function useMarkerManager(machineId: string) {
  const markers = useMarkerStore((state) => {
    const list = state.markersByMachine[machineId];
    return list ?? EMPTY_MARKERS;
  });
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
