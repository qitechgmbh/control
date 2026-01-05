# IndexedDB-Based Timeseries System

## Overview

This system replaces the previous RAM-hungry timeseries handling with an efficient IndexedDB-based approach. Data points are now automatically persisted to the browser's IndexedDB storage through namespace event listeners, allowing for both historical and live data to be displayed in charts without consuming excessive memory.

## Architecture

### Components

1. **IndexedDB Storage Layer** (`indexedDBTimeseries.ts`)
   - Manages persistent storage of timeseries data points
   - Provides efficient querying by series and time range
   - Handles batch operations for performance

2. **Writer Service** (`timeseriesWriter.ts`)
   - Batches data points for efficient writes
   - Manages namespace-specific writers
   - Automatic flushing based on time/size thresholds

3. **Reader Hook** (`useTimeSeriesData.ts`)
   - React hook for reading timeseries data from IndexedDB
   - Supports time windows and point limits
   - Auto-refresh for live updates

4. **Cleanup Service** (`timeseriesCleanup.ts`)
   - Automatically removes old data points
   - Configurable retention periods
   - Periodic cleanup runs

5. **Initialization** (`initTimeseries.ts`)
   - Bootstraps the entire system
   - Manages lifecycle (startup/shutdown)

## How It Works

### Data Flow

```
Backend → Namespace Event → Writer (batching) → IndexedDB → Reader Hook → Chart
```

1. **Backend sends events** through Socket.IO namespaces
2. **Namespace handler** receives events and extracts timeseries data
3. **Writer service** batches the data points
4. **IndexedDB** persists the data points with indexes
5. **Reader hook** queries data for charts
6. **Cleanup service** removes old data periodically

### Key Features

- **Memory Efficient**: Data stored in IndexedDB, not RAM
- **Batched Writes**: Reduces IndexedDB write overhead
- **Fast Queries**: Compound indexes for efficient range queries
- **Automatic Cleanup**: Configurable retention periods
- **Live Updates**: Charts refresh automatically
- **Historical Data**: Access old data without keeping it in memory

## Usage

### 1. Setting Up Namespace Data Writing

In your namespace handler, add timeseries writing:

```typescript
import { getWriterForNamespace, generateSeriesKey } from "@/lib/timeseriesWriter";

export function myNamespaceMessageHandler(
  store: StoreApi<MyNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MyNamespaceStore>,
  namespaceId: NamespaceId,
): EventHandler {
  // Get writer for this namespace
  const namespaceKey =
    namespaceId.type === "machine"
      ? `myMachine:${namespaceId.machine_identification_unique.serial}`
      : "myMachine:main";
  const writer = getWriterForNamespace(namespaceKey);

  return (event: Event<any>) => {
    if (event.name === "LiveValuesEvent") {
      const timestamp = event.ts;
      const value = event.data.measurement;

      // Write to IndexedDB
      if (namespaceId.type === "machine") {
        const serial = namespaceId.machine_identification_unique.serial;
        writer.write(
          generateSeriesKey("myMachine", serial, "measurement"),
          timestamp,
          value,
        );
      }

      // Update store for live display
      updateStore((state) => ({ ...state, measurement: value }));
    }
  };
}
```

### 2. Reading Data in Components

Use the `useTimeSeriesData` hook:

```typescript
import { useTimeSeriesData } from "@/lib/useTimeSeriesData";
import { generateSeriesKey } from "@/lib/timeseriesWriter";

function MyChart({ serial }: { serial: number }) {
  const seriesKey = generateSeriesKey("myMachine", serial, "measurement");

  const { timestamps, values, isLoading, latest } = useTimeSeriesData({
    seriesKey,
    timeWindowMs: 60 * 60 * 1000, // 1 hour
    refreshIntervalMs: 100, // Refresh every 100ms
  });

  return (
    <Chart data={{ timestamps, values }} loading={isLoading} />
  );
}
```

### 3. Multiple Series

For charts with multiple series:

```typescript
import { useMultipleTimeSeries } from "@/lib/useTimeSeriesData";

const { data, isLoading } = useMultipleTimeSeries({
  series: [
    { key: "temp1", seriesKey: generateSeriesKey("machine", serial, "temp1") },
    { key: "temp2", seriesKey: generateSeriesKey("machine", serial, "temp2") },
  ],
  timeWindowMs: 5000, // 5 seconds
  refreshIntervalMs: 100,
});

const temp1Data = data.get("temp1");
const temp2Data = data.get("temp2");
```

## Configuration

### Writer Configuration

```typescript
const writer = new TimeSeriesWriter({
  batchFlushInterval: 100, // Flush every 100ms
  maxBatchSize: 50, // Flush after 50 points
  enableBatching: true,
});
```

### Cleanup Configuration

```typescript
startCleanupService({
  retentionPeriodMs: 24 * 60 * 60 * 1000, // Keep data for 24 hours
  cleanupIntervalMs: 60 * 60 * 1000, // Run cleanup every hour
  enabled: true,
});
```

## Series Key Format

Series keys follow the format: `{machineType}:{serial}:{fieldName}`

Examples:
- `mock1:123:amplitude1`
- `extruder:456:temperature`
- `laser:789:power`

This format ensures uniqueness across machines and data fields.

## Database Schema

### Object Store: `datapoints`

- **Primary Key**: `id` (composite: `seriesKey:timestamp`)
- **Fields**:
  - `id`: string
  - `seriesKey`: string
  - `timestamp`: number (ms)
  - `value`: number

### Indexes

1. **by-series**: Index on `seriesKey` for finding all points for a series
2. **by-series-time**: Compound index on `[seriesKey, timestamp]` for range queries
3. **by-timestamp**: Index on `timestamp` for cleanup operations

## Performance Considerations

### Write Performance

- Batching reduces IndexedDB transactions
- Default batch size: 50 points
- Default flush interval: 100ms
- Typical throughput: >1000 points/second

### Read Performance

- Compound indexes enable fast range queries
- Query 1 hour of data (at 50Hz): ~5-10ms
- Auto-refresh every 100ms: minimal overhead

### Storage

- IndexedDB quota: Browser-dependent (typically 50-100MB)
- 24-hour retention at 50Hz: ~4-5 million points
- Approximate size: 40-50MB
- Cleanup service manages quota automatically

## Migration Guide

### From Old RAM-Based System

**Before:**
```typescript
const timeSeries = createTimeSeries({ ... });
const [seriesData, setSeriesData] = useState(timeSeries.initialTimeSeries);

useEffect(() => {
  const dataPoint = { value, timestamp };
  setSeriesData(timeSeries.insert(seriesData, dataPoint));
}, [measurement]);
```

**After:**
```typescript
// In namespace handler (automatic writing)
writer.write(seriesKey, timestamp, value);

// In component (automatic reading)
const { timestamps, values } = useTimeSeriesData({
  seriesKey,
  timeWindowMs: 60 * 60 * 1000,
});
```

## Troubleshooting

### Data Not Appearing

1. Check that namespace handler is writing data
2. Verify series key matches between writer and reader
3. Check browser console for IndexedDB errors

### Performance Issues

1. Reduce refresh interval (default: 100ms)
2. Limit time window for queries
3. Check cleanup service is running

### Storage Quota Exceeded

1. Reduce retention period
2. Increase cleanup frequency
3. Clear old data manually: `clearAllData()`

## API Reference

See individual file documentation:
- [`indexedDBTimeseries.ts`](./indexedDBTimeseries.ts)
- [`timeseriesWriter.ts`](./timeseriesWriter.ts)
- [`useTimeSeriesData.ts`](./useTimeSeriesData.ts)
- [`timeseriesCleanup.ts`](./timeseriesCleanup.ts)

## Examples

Complete examples can be found in:
- [mock1Namespace.ts](../machines/mock/mock1/mock1Namespace.ts)
- [useAnalogInputTestMachine.ts](../machines/analoginputtestmachine/useAnalogInputTestMachine.ts)
