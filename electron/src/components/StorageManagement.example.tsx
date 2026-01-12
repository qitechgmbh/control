/**
 * Storage Management Component - Example Implementation
 * 
 * This component demonstrates how to integrate the IndexedDB storage
 * management system into your UI. It can be used as-is in a settings
 * page or adapted to your specific needs.
 * 
 * Features:
 * - Display storage usage and quota
 * - Show series statistics
 * - Export data to CSV/JSON
 * - Clear old data
 * - Request persistent storage
 */

import React from "react";
import { useStorageDashboard } from "@/hooks/useStorageManagement";
import { serializeNamespaceId, NamespaceId } from "@/client/socketioStore";

interface StorageManagementProps {
  /** Optional namespace to manage */
  namespaceId?: NamespaceId;
  /** Optional series name within the namespace */
  seriesName?: string;
}

export function StorageManagement({
  namespaceId,
  seriesName,
}: StorageManagementProps) {
  const serializedNamespaceId = namespaceId
    ? serializeNamespaceId(namespaceId)
    : undefined;

  const {
    // Storage usage
    usage,
    usageError,

    // Capabilities
    available,
    persisted,
    requesting,
    requestPersistence,

    // Formatting
    formatStorageSize,
    formatPercentage,
    formatTimestamp,

    // Series stats
    seriesStats,
    seriesLoading,
    seriesError,
    refreshSeriesStats,

    // Series management
    clearSeries,
    exportCSV,
    exportJSON,
    clearingSeriesData,

    // Namespace management
    clearNamespace,
    clearingNamespaceData,
  } = useStorageDashboard(serializedNamespaceId, seriesName);

  if (!available) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4">
        <h3 className="mb-2 text-lg font-semibold text-red-900">
          Storage Not Available
        </h3>
        <p className="text-red-700">
          IndexedDB storage is not supported in this browser. Chart data will
          only be stored in memory and will be lost on navigation.
        </p>
      </div>
    );
  }

  const handleRequestPersistence = async () => {
    const granted = await requestPersistence();
    if (granted) {
      alert("Persistent storage granted! Your data is now protected.");
    } else {
      alert(
        "Persistent storage denied. Data may be cleared by the browser when storage is low.",
      );
    }
  };

  const handleExportCSV = async () => {
    if (!exportCSV) return;
    try {
      await exportCSV();
      alert("Data exported successfully!");
    } catch (error) {
      alert(`Export failed: ${error}`);
    }
  };

  const handleExportJSON = async () => {
    if (!exportJSON) return;
    try {
      await exportJSON();
      alert("Data exported successfully!");
    } catch (error) {
      alert(`Export failed: ${error}`);
    }
  };

  const handleClearSeries = async () => {
    if (!clearSeries) return;
    const confirmed = confirm(
      "Are you sure you want to clear all data for this series? This cannot be undone.",
    );
    if (confirmed) {
      try {
        await clearSeries();
        refreshSeriesStats?.();
        alert("Series data cleared successfully!");
      } catch (error) {
        alert(`Clear failed: ${error}`);
      }
    }
  };

  const handleClearNamespace = async () => {
    if (!clearNamespace) return;
    const confirmed = confirm(
      "Are you sure you want to clear ALL data for this namespace? This cannot be undone.",
    );
    if (confirmed) {
      try {
        await clearNamespace();
        refreshSeriesStats?.();
        alert("Namespace data cleared successfully!");
      } catch (error) {
        alert(`Clear failed: ${error}`);
      }
    }
  };

  return (
    <div className="space-y-6">
      {/* Overall Storage Usage */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="mb-4 text-xl font-semibold">Storage Usage</h2>

        {usageError ? (
          <p className="text-red-600">Error loading storage info: {usageError.message}</p>
        ) : !usage ? (
          <p className="text-gray-500">Loading...</p>
        ) : (
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-700">Used:</span>
              <span className="font-mono font-medium">
                {formatStorageSize(usage.usage)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-700">Quota:</span>
              <span className="font-mono font-medium">
                {formatStorageSize(usage.quota)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-700">Percent Used:</span>
              <span className="font-mono font-medium">
                {formatPercentage(usage.percentUsed)}
              </span>
            </div>

            {/* Visual progress bar */}
            <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-gray-200">
              <div
                className={`h-full transition-all ${
                  usage.percentUsed > 90
                    ? "bg-red-500"
                    : usage.percentUsed > 70
                      ? "bg-yellow-500"
                      : "bg-green-500"
                }`}
                style={{ width: `${Math.min(usage.percentUsed, 100)}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* Persistence Status */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="mb-4 text-xl font-semibold">Persistence Status</h2>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-gray-700">Protected from browser cleanup:</span>
            <span
              className={`rounded px-2 py-1 text-sm font-medium ${
                persisted
                  ? "bg-green-100 text-green-800"
                  : "bg-yellow-100 text-yellow-800"
              }`}
            >
              {persisted ? "Yes" : "No"}
            </span>
          </div>

          {!persisted && (
            <button
              onClick={handleRequestPersistence}
              disabled={requesting}
              className="w-full rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:opacity-50"
            >
              {requesting ? "Requesting..." : "Request Persistent Storage"}
            </button>
          )}

          <p className="text-sm text-gray-600">
            {persisted
              ? "Your data is protected and won't be cleared by the browser automatically."
              : "Your data may be cleared if browser storage is low. Click to request protection."}
          </p>
        </div>
      </div>

      {/* Series-Specific Management */}
      {seriesName && serializedNamespaceId && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
          <h2 className="mb-4 text-xl font-semibold">
            Series: {seriesName}
          </h2>

          {seriesError ? (
            <p className="text-red-600">Error: {seriesError.message}</p>
          ) : seriesLoading ? (
            <p className="text-gray-500">Loading series stats...</p>
          ) : !seriesStats ? (
            <p className="text-gray-500">No data available</p>
          ) : (
            <>
              <div className="mb-4 space-y-2">
                <div className="flex justify-between">
                  <span className="text-gray-700">Data Points:</span>
                  <span className="font-mono">{seriesStats.count.toLocaleString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-700">Estimated Size:</span>
                  <span className="font-mono">
                    {formatStorageSize(seriesStats.estimatedSizeBytes)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-700">Oldest Data:</span>
                  <span className="font-mono text-sm">
                    {formatTimestamp(seriesStats.oldestTimestamp)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-700">Newest Data:</span>
                  <span className="font-mono text-sm">
                    {formatTimestamp(seriesStats.newestTimestamp)}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <button
                  onClick={refreshSeriesStats}
                  className="w-full rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50"
                >
                  Refresh Stats
                </button>

                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={handleExportCSV}
                    className="rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50"
                  >
                    Export CSV
                  </button>
                  <button
                    onClick={handleExportJSON}
                    className="rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50"
                  >
                    Export JSON
                  </button>
                </div>

                <button
                  onClick={handleClearSeries}
                  disabled={clearingSeriesData}
                  className="w-full rounded bg-red-600 px-4 py-2 text-white hover:bg-red-700 disabled:opacity-50"
                >
                  {clearingSeriesData ? "Clearing..." : "Clear Series Data"}
                </button>
              </div>
            </>
          )}
        </div>
      )}

      {/* Namespace-Level Management */}
      {serializedNamespaceId && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-6 shadow-sm">
          <h2 className="mb-4 text-xl font-semibold text-red-900">
            Danger Zone
          </h2>
          <p className="mb-4 text-sm text-red-700">
            Clear all timeseries data for this namespace. This action cannot be undone.
          </p>
          <button
            onClick={handleClearNamespace}
            disabled={clearingNamespaceData}
            className="w-full rounded bg-red-700 px-4 py-2 text-white hover:bg-red-800 disabled:opacity-50"
          >
            {clearingNamespaceData
              ? "Clearing..."
              : "Clear All Namespace Data"}
          </button>
        </div>
      )}
    </div>
  );
}

/**
 * Example usage in a settings page:
 * 
 * ```tsx
 * import { StorageManagement } from '@/components/StorageManagement';
 * 
 * function SettingsPage() {
 *   return (
 *     <div className="p-8">
 *       <h1>Storage Settings</h1>
 *       <StorageManagement />
 *     </div>
 *   );
 * }
 * ```
 * 
 * Example with specific namespace and series:
 * 
 * ```tsx
 * function Winder2Settings({ machineId }) {
 *   const namespaceId = {
 *     type: 'machine',
 *     machine_identification_unique: machineId
 *   };
 *   
 *   return (
 *     <StorageManagement 
 *       namespaceId={namespaceId}
 *       seriesName="pullerSpeed"
 *     />
 *   );
 * }
 * ```
 */
