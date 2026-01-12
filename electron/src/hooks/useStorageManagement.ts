/**
 * React hooks for IndexedDB storage management
 * 
 * Provides easy-to-use hooks for React components to interact with
 * the persistent timeseries storage system.
 */

import { useState, useEffect, useCallback } from "react";
import {
  getStorageUsage,
  getNamespaceSeriesStats,
  clearSeries,
  clearNamespace,
  downloadSeriesAsCSV,
  downloadSeriesAsJSON,
  isPersistenceAvailable,
  isStoragePersisted,
  requestPersistentStorage,
  formatBytes,
} from "@/lib/storageManagement";

/**
 * Hook to monitor storage usage
 * Updates every 5 seconds by default
 */
export function useStorageUsage(updateInterval: number = 5000) {
  const [usage, setUsage] = useState<{
    usage: number;
    quota: number;
    percentUsed: number;
  } | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const updateUsage = async () => {
      try {
        const result = await getStorageUsage();
        setUsage(result);
        setError(null);
      } catch (err) {
        setError(err as Error);
      }
    };

    updateUsage();
    const interval = setInterval(updateUsage, updateInterval);

    return () => clearInterval(interval);
  }, [updateInterval]);

  return { usage, error };
}

/**
 * Hook to get statistics for a specific series
 */
export function useSeriesStats(namespaceId: string, seriesName: string) {
  const [stats, setStats] = useState<{
    count: number;
    oldestTimestamp: number | null;
    newestTimestamp: number | null;
    estimatedSizeBytes: number;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await getNamespaceSeriesStats(namespaceId, seriesName);
      setStats(result);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [namespaceId, seriesName]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { stats, loading, error, refresh };
}

/**
 * Hook to manage series data clearing
 */
export function useSeriesManagement(namespaceId: string, seriesName: string) {
  const [clearing, setClearing] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const clear = useCallback(async () => {
    setClearing(true);
    setError(null);
    try {
      const success = await clearSeries(namespaceId, seriesName);
      if (!success) {
        throw new Error("Failed to clear series data");
      }
    } catch (err) {
      setError(err as Error);
      throw err;
    } finally {
      setClearing(false);
    }
  }, [namespaceId, seriesName]);

  const exportCSV = useCallback(
    async (startTime?: number, endTime?: number) => {
      try {
        await downloadSeriesAsCSV(namespaceId, seriesName, startTime, endTime);
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    [namespaceId, seriesName],
  );

  const exportJSON = useCallback(
    async (startTime?: number, endTime?: number) => {
      try {
        await downloadSeriesAsJSON(namespaceId, seriesName, startTime, endTime);
      } catch (err) {
        setError(err as Error);
        throw err;
      }
    },
    [namespaceId, seriesName],
  );

  return {
    clear,
    exportCSV,
    exportJSON,
    clearing,
    error,
  };
}

/**
 * Hook to manage namespace-level data
 */
export function useNamespaceManagement(namespaceId: string) {
  const [clearing, setClearing] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const clearAll = useCallback(async () => {
    setClearing(true);
    setError(null);
    try {
      const success = await clearNamespace(namespaceId);
      if (!success) {
        throw new Error("Failed to clear namespace data");
      }
    } catch (err) {
      setError(err as Error);
      throw err;
    } finally {
      setClearing(false);
    }
  }, [namespaceId]);

  return {
    clearAll,
    clearing,
    error,
  };
}

/**
 * Hook to check persistence capabilities
 */
export function usePersistenceCapabilities() {
  const [available, setAvailable] = useState(false);
  const [persisted, setPersisted] = useState(false);
  const [requesting, setRequesting] = useState(false);

  useEffect(() => {
    setAvailable(isPersistenceAvailable());

    isStoragePersisted().then(setPersisted);
  }, []);

  const requestPersistence = useCallback(async () => {
    setRequesting(true);
    try {
      const granted = await requestPersistentStorage();
      setPersisted(granted);
      return granted;
    } finally {
      setRequesting(false);
    }
  }, []);

  return {
    available,
    persisted,
    requesting,
    requestPersistence,
  };
}

/**
 * Hook to format storage values for display
 */
export function useStorageFormatting() {
  const formatStorageSize = useCallback((bytes: number) => {
    return formatBytes(bytes);
  }, []);

  const formatPercentage = useCallback((percent: number) => {
    return `${percent.toFixed(1)}%`;
  }, []);

  const formatTimestamp = useCallback((timestamp: number | null) => {
    if (timestamp === null) return "N/A";
    return new Date(timestamp).toLocaleString();
  }, []);

  return {
    formatStorageSize,
    formatPercentage,
    formatTimestamp,
  };
}

/**
 * Combined hook for storage dashboard
 */
export function useStorageDashboard(namespaceId?: string, seriesName?: string) {
  const { usage, error: usageError } = useStorageUsage();
  const capabilities = usePersistenceCapabilities();
  const formatting = useStorageFormatting();

  const seriesStats =
    namespaceId && seriesName
      ? // eslint-disable-next-line react-hooks/rules-of-hooks
        useSeriesStats(namespaceId, seriesName)
      : null;

  const seriesManagement =
    namespaceId && seriesName
      ? // eslint-disable-next-line react-hooks/rules-of-hooks
        useSeriesManagement(namespaceId, seriesName)
      : null;

  const namespaceManagement = namespaceId
    ? // eslint-disable-next-line react-hooks/rules-of-hooks
      useNamespaceManagement(namespaceId)
    : null;

  return {
    // Storage usage
    usage,
    usageError,

    // Capabilities
    ...capabilities,

    // Formatting
    ...formatting,

    // Series-specific (if provided)
    seriesStats: seriesStats?.stats,
    seriesLoading: seriesStats?.loading,
    seriesError: seriesStats?.error,
    refreshSeriesStats: seriesStats?.refresh,

    // Series management (if provided)
    clearSeries: seriesManagement?.clear,
    exportCSV: seriesManagement?.exportCSV,
    exportJSON: seriesManagement?.exportJSON,
    clearingSeriesData: seriesManagement?.clearing,

    // Namespace management (if provided)
    clearNamespace: namespaceManagement?.clearAll,
    clearingNamespaceData: namespaceManagement?.clearing,
  };
}
