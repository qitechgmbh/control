import { useState, useCallback, useEffect } from "react";

export type Marker = {
  timestamp: number;
  name: string;
  value?: number; // Optional: value at that timestamp
  color?: string; // Optional: color for the marker
};

// Custom event for marker updates to ensure immediate propagation
const MARKER_UPDATE_EVENT = "marker-update";

/**
 * Centralized marker management for all graphs of a machine
 * Markers are stored per machine (machineId) and appear on all graphs
 */
export function useMarkerManager(machineId: string) {
  const storageKey = `machine-markers-${machineId}`;

  // Load markers from localStorage (no time-based deletion; keep last 200 only)
  const loadMarkers = useCallback((): Marker[] => {
    try {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const allMarkers: Marker[] = JSON.parse(stored);
        const maxMarkers = 200;
        const limited =
          allMarkers.length > maxMarkers
            ? allMarkers.slice(-maxMarkers)
            : allMarkers;
        if (limited.length !== allMarkers.length) {
          localStorage.setItem(storageKey, JSON.stringify(limited));
        }
        return limited;
      }
    } catch (error) {
      console.warn("Failed to load markers from localStorage:", error);
    }
    return [];
  }, [storageKey]);

  const [markers, setMarkers] = useState<Marker[]>(loadMarkers);

  // Listen for marker updates from other components
  useEffect(() => {
    const handleMarkerUpdate = (event: CustomEvent) => {
      if (event.detail?.machineId === machineId) {
        // Reload markers immediately when updated
        setMarkers(loadMarkers());
      }
    };

    window.addEventListener(
      MARKER_UPDATE_EVENT,
      handleMarkerUpdate as EventListener,
    );

    return () => {
      window.removeEventListener(
        MARKER_UPDATE_EVENT,
        handleMarkerUpdate as EventListener,
      );
    };
  }, [machineId, loadMarkers]);

  // Save markers to localStorage whenever they change
  useEffect(() => {
    try {
      const maxMarkers = 200;
      const markersToSave =
        markers.length > maxMarkers ? markers.slice(-maxMarkers) : markers;

      localStorage.setItem(storageKey, JSON.stringify(markersToSave));
    } catch (error) {
      console.warn("Failed to save markers to localStorage:", error);
    }
  }, [markers, storageKey]);

  const addMarker = useCallback(
    (name: string, timestamp: number, color?: string, value?: number) => {
      const newMarker: Marker = {
        timestamp,
        name,
        color,
        value,
      };
      setMarkers((prev) => {
        const updated = [...prev, newMarker];
        // Save immediately to localStorage
        try {
          const maxMarkers = 200;
          const markersToSave =
            updated.length > maxMarkers ? updated.slice(-maxMarkers) : updated;
          localStorage.setItem(storageKey, JSON.stringify(markersToSave));
        } catch (error) {
          console.warn("Failed to save marker to localStorage:", error);
        }
        // Dispatch event to notify other components immediately
        window.dispatchEvent(
          new CustomEvent(MARKER_UPDATE_EVENT, {
            detail: { machineId, markers: updated },
          }),
        );
        return updated;
      });
      return newMarker;
    },
    [storageKey, machineId],
  );

  const removeMarker = useCallback(
    (timestamp: number) => {
      setMarkers((prev) => {
        const updated = prev.filter((marker) => marker.timestamp !== timestamp);
        // Persist and notify other components (e.g. dialog, other graphs)
        try {
          localStorage.setItem(storageKey, JSON.stringify(updated));
          window.dispatchEvent(
            new CustomEvent(MARKER_UPDATE_EVENT, {
              detail: { machineId, markers: updated },
            }),
          );
        } catch (error) {
          console.warn("Failed to save markers after remove:", error);
        }
        return updated;
      });
    },
    [storageKey, machineId],
  );

  const clearMarkers = useCallback(() => {
    setMarkers([]);
  }, []);

  return {
    markers,
    addMarker,
    removeMarker,
    clearMarkers,
  };
}
