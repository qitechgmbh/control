# IndexedDB Chart Data Persistence

This document describes the IndexedDB-based persistence layer for chart/timeseries data in the QiTech Control application.

## Overview

The application now supports persistent storage of live chart data using IndexedDB. This system:

- **Persists long timeseries data** (1h+ retention) to IndexedDB automatically
- **Keeps short timeseries** (5s retention) in RAM only for performance  
- **Preserves data** across page navigation and browser sessions
- **Maintains API compatibility** with existing namespace implementations
- **Loads historical data** on demand for visible time windows

## Architecture

### Key Components

1. **`timeseriesDB.ts`**: Low-level IndexedDB wrapper
   - Handles all database operations (create, read, delete)
   - Provides efficient time-range queries
   - Manages data cleanup and retention policies

2. **`timeseriesPersistent.ts`**: Persistent TimeSeries implementation
   - Drop-in replacement for `createTimeSeries()`
   - Automatically persists long-series data points
   - Loads historical data on initialization
   - Maintains circular buffer in RAM for performance

3. **`namespacePersistence.ts`**: Namespace integration helpers
   - Simplifies creating multiple persistent series
   - Provides type-safe APIs for namespace stores
   - Handles async initialization gracefully

4. **`winder2NamespaceHybrid.example.ts`**: Migration example
   - Shows how to migrate existing namespaces
   - Demonstrates best practices
   - Includes detailed migration notes

### Data Flow

```
Socket.io Event → Namespace Handler → Insert Function → 
  ├─ Short Series (RAM only)
  └─ Long Series (RAM + IndexedDB)
      ├─ Update circular buffer
      └─ Persist to IndexedDB (async)

Page Load → Initialize Store →
  └─ Load Historical Data from IndexedDB (async) →
      └─ Populate Long Series Buffer
```

### IndexedDB Schema

**Database**: `QiTechControlTimeSeries`

**Store**: `timeseries_data`
- **Key**: `[namespaceId, seriesName, timestamp]` (composite)
- **Indexes**:
  - `by-namespace`: Query all series for a namespace
  - `by-namespace-series`: Query specific series
  - `by-namespace-series-time`: Efficient time-range queries

**Data Model**:
```typescript
{
  namespaceId: string,    // Serialized NamespaceId
  seriesName: string,     // e.g., "pullerSpeed"
  timestamp: number,      // Unix timestamp in ms
  value: number          // Data point value
}
```

## Usage

### Option 1: Quick Migration (Recommended)

Replace `createTimeSeries()` with `createPersistentTimeSeries()`:

```typescript
// Before
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } = 
  createTimeSeries();

// After
const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } = 
  createPersistentTimeSeries(
    serializeNamespaceId(namespaceId),
    "pullerSpeed"
  );
```

### Option 2: Namespace-Level Integration

Use the helper for multiple series:

```typescript
import { createNamespacePersistentTimeSeries } from "@/lib/namespacePersistence";

export const createWinder2NamespaceStore = (
  namespaceId: NamespaceId
): StoreApi<Winder2NamespaceStore> => {
  const seriesResult = createNamespacePersistentTimeSeries(
    namespaceId,
    {
      traversePosition: "traversePosition",
      pullerSpeed: "pullerSpeed",
      spoolRpm: "spoolRpm",
      tensionArmAngle: "tensionArmAngle",
      spoolProgress: "spoolProgress",
    }
  );

  const store = create<Winder2NamespaceStore>(() => ({
    state: null,
    defaultState: null,
    ...seriesResult.initialState,
  }));

  // Load historical data
  seriesResult.ready.then(historicalSeries => {
    store.setState(historicalSeries);
  });

  return store;
};
```

### Option 3: Custom Configuration

Override default retention and cleanup settings:

```typescript
const { initialTimeSeries, insert } = createPersistentTimeSeries(
  namespaceId,
  "seriesName",
  {
    enablePersistence: true,
    maxRetentionTime: 14 * 24 * 60 * 60 * 1000, // 14 days
    cleanupInterval: 2 * 60 * 60 * 1000,        // 2 hours
    retentionDurationLong: 2 * 60 * 60 * 1000,  // 2h buffer
  }
);
```

## Migration Guide

### Step 1: Identify Target Namespaces

Identify which namespaces should use persistence:
- Namespaces with important historical data
- Namespaces where users navigate away and back
- High-frequency data that benefits from retention

### Step 2: Update Store Creation

Modify the `createXNamespaceStore` function:

```typescript
// Add namespaceId parameter
export const createWinder2NamespaceStore = (
  namespaceId: NamespaceId  // ← Add this
): StoreApi<Winder2NamespaceStore> => {
  // Use createNamespacePersistentTimeSeries
  const seriesResult = createNamespacePersistentTimeSeries(
    namespaceId,
    { /* series definitions */ }
  );
  
  // Store with initial state
  const store = create(() => ({
    ...seriesResult.initialState,
  }));
  
  // Load historical data
  seriesResult.ready.then(data => store.setState(data));
  
  return store;
};
```

### Step 3: Update Message Handler

Modify the message handler to use insert functions from `seriesResult`:

```typescript
export function winder2MessageHandler(
  namespaceId: NamespaceId,  // ← Add this
  store: StoreApi<Winder2NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Winder2NamespaceStore>,
): EventHandler {
  const insertFns = getSeriesInsertFunctions(namespaceId);
  
  return (event: Event<any>) => {
    // Use insertFns.seriesName instead of addSeriesName
    updateStore((state) => ({
      ...state,
      pullerSpeed: insertFns.pullerSpeed(
        state.pullerSpeed,
        newValue
      ),
    }));
  };
}
```

### Step 4: Update Hook Implementation

Pass namespaceId through to the store creator:

```typescript
export function useWinder2Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Winder2NamespaceStore {
  const namespaceId = useMemo<NamespaceId>(() => ({
    type: "machine",
    machine_identification_unique,
  }), [machine_identification_unique]);

  const useImpl = useMemo(() => {
    return createNamespaceHookImplementation<Winder2NamespaceStore>({
      createStore: () => createWinder2NamespaceStore(namespaceId),  // ← Pass ID
      createEventHandler: (store, updater) =>
        winder2MessageHandler(namespaceId, store, updater),  // ← Pass ID
    });
  }, [namespaceId]);

  return useImpl(namespaceId);
}
```

### Step 5: Test

1. **Verify data persistence**: Navigate away and back, check that charts retain data
2. **Check performance**: Ensure no noticeable slowdown
3. **Test with large datasets**: Generate hours of data
4. **Monitor IndexedDB**: Use browser DevTools → Application → IndexedDB
5. **Test cleanup**: Verify old data is removed according to retention policy

## API Reference

### `createPersistentTimeSeries(namespaceId, seriesName, config?)`

Creates a persistent timeseries.

**Parameters**:
- `namespaceId: string` - Serialized namespace identifier
- `seriesName: string` - Name of the series (e.g., "pullerSpeed")
- `config?: Partial<PersistentTimeSeriesConfig>` - Optional configuration

**Returns**:
```typescript
{
  initialTimeSeries: TimeSeries,
  insert: (series: TimeSeries, value: TimeSeriesValue) => TimeSeries,
  ready: Promise<TimeSeries>,  // Resolves when historical data is loaded
  cleanup: () => Promise<void>  // Manually trigger cleanup
}
```

### `createNamespacePersistentTimeSeries(namespaceId, seriesDefinitions, config?)`

Creates multiple persistent series for a namespace.

**Parameters**:
- `namespaceId: NamespaceId` - Namespace identifier
- `seriesDefinitions: Record<string, string>` - Map of series keys to names
- `config?: Partial<PersistentTimeSeriesConfig>` - Global configuration

**Returns**:
```typescript
{
  initialState: Record<string, TimeSeries>,
  insertFns: Record<string, InsertFunction>,
  ready: Promise<Record<string, TimeSeries>>,
  cleanup: () => Promise<void>
}
```

### Configuration Options

```typescript
interface PersistentTimeSeriesConfig {
  // From TimeSeriesConfig
  sampleIntervalShort: number;      // Default: 20ms
  sampleIntervalLong: number;       // Default: 1000ms (1s)
  retentionDurationShort: number;   // Default: 5000ms (5s)
  retentionDurationLong: number;    // Default: 3600000ms (1h)
  
  // Persistence-specific
  enablePersistence: boolean;       // Default: true
  maxRetentionTime: number;         // Default: 7 days
  cleanupInterval: number;          // Default: 1 hour
}
```

## Performance Considerations

### Memory Usage

- **Short series**: ~250 points × 8 bytes/point = ~2KB per series
- **Long series**: ~3600 points × 8 bytes/point = ~28KB per series
- **Per namespace**: ~5 series × 30KB = ~150KB per machine instance

### IndexedDB Storage

- **Per data point**: ~40 bytes (including keys and metadata)
- **1 hour of data**: 3600 points × 40 bytes × 5 series = ~720KB per machine
- **7 days retention**: ~120MB per machine
- **Cleanup**: Automatic, runs every hour

### Query Performance

- **Time-range queries**: O(log n) with indexed lookups
- **Latest N points**: O(n) scan required
- **Typical load time**: <100ms for 1 hour of data

## Troubleshooting

### Data not persisting

1. Check browser's IndexedDB quota (DevTools → Application → Storage)
2. Verify `enablePersistence: true` in config
3. Check console for errors during `storeDataPoint()` calls

### Slow initial load

1. Reduce `maxRetentionTime` to limit historical data
2. Check for large number of series in namespace
3. Consider implementing pagination for very long time ranges

### Memory issues

1. Monitor `validCount` in series buffers
2. Check that cleanup is running (watch console logs)
3. Consider reducing `retentionDurationLong`

### IndexedDB quota exceeded

1. Reduce `maxRetentionTime` across all namespaces
2. Implement manual data export/clear functionality
3. Run cleanup more frequently (reduce `cleanupInterval`)

## Best Practices

1. **Use consistent series names**: They're used as database keys
2. **Implement loading indicators**: Show users when historical data is loading
3. **Handle the ready promise**: Don't assume data is immediately available
4. **Monitor storage usage**: Add UI to show used storage
5. **Provide clear data**: Add buttons for users to manually clear old data
6. **Test offline behavior**: Ensure graceful degradation when IndexedDB fails
7. **Version your data format**: Plan for schema migrations

## Future Enhancements

- **Data compression**: Use compression for long-term storage
- **Selective persistence**: Allow users to choose which series to persist
- **Cross-tab synchronization**: Sync data across multiple browser tabs
- **Export/import**: Allow users to export data as CSV/JSON
- **Cloud sync**: Optionally sync data to cloud storage
- **Intelligent loading**: Load only visible time windows on demand
- **Progressive loading**: Load recent data first, then backfill

## See Also

- [winder2NamespaceHybrid.example.ts](../electron/src/machines/winder/winder2/winder2NamespaceHybrid.example.ts) - Complete migration example
- [timeseries.ts](../electron/src/lib/timeseries.ts) - Original RAM-only implementation
- [timeseriesDB.ts](../electron/src/lib/timeseriesDB.ts) - IndexedDB wrapper
- [timeseriesPersistent.ts](../electron/src/lib/timeseriesPersistent.ts) - Persistent implementation
- [namespacePersistence.ts](../electron/src/lib/namespacePersistence.ts) - Namespace helpers
