# Migration Guide: Updating Namespaces to Use IndexedDB Timeseries

This guide explains how to update existing namespace handlers to use the new IndexedDB-based timeseries system.

## Step-by-Step Migration

### 1. Update Namespace Handler Signature

Add the `namespaceId` parameter to your message handler:

**Before:**
```typescript
export function myNamespaceMessageHandler(
  store: StoreApi<MyNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MyNamespaceStore>,
): EventHandler {
```

**After:**
```typescript
export function myNamespaceMessageHandler(
  store: StoreApi<MyNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MyNamespaceStore>,
  namespaceId: NamespaceId,
): EventHandler {
```

### 2. Add Writer Setup

Import the writer utilities and create a writer instance:

```typescript
import {
  getWriterForNamespace,
  generateSeriesKey,
} from "@/lib/timeseriesWriter";

export function myNamespaceMessageHandler(
  store: StoreApi<MyNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MyNamespaceStore>,
  namespaceId: NamespaceId,
): EventHandler {
  // Get writer for this namespace
  const namespaceKey =
    namespaceId.type === "machine"
      ? `myMachineName:${namespaceId.machine_identification_unique.serial}`
      : "myMachineName:main";
  const writer = getWriterForNamespace(namespaceKey);

  return (event: Event<any>) => {
    // ... handler logic
  };
}
```

### 3. Write Timeseries Data to IndexedDB

For each timeseries data point in your event handler:

**Before:**
```typescript
const dataPoint: TimeSeriesValue = {
  value: event.data.measurement,
  timestamp: event.ts,
};
updateStore((state) => ({
  ...state,
  measurement: addMeasurement(state.measurement, dataPoint),
}));
```

**After:**
```typescript
const timestamp = event.ts;
const value = event.data.measurement;

// Write to IndexedDB
if (namespaceId.type === "machine") {
  const serial = namespaceId.machine_identification_unique.serial;
  writer.write(
    generateSeriesKey("myMachineName", serial, "measurement"),
    timestamp,
    value,
  );
}

// Still update store for current value display (optional)
updateStore((state) => ({
  ...state,
  currentMeasurement: value,
}));
```

### 4. Update Component/Hook to Read from IndexedDB

**Before:**
```typescript
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

const timeSeries = createTimeSeries({ ... });
const [seriesData, setSeriesData] = useState(timeSeries.initialTimeSeries);

useEffect(() => {
  // Manual data point insertion
  const dataPoint = { value, timestamp };
  setSeriesData(timeSeries.insert(seriesData, dataPoint));
}, [measurement]);

return {
  seriesData, // TimeSeries object
  // ...
};
```

**After:**
```typescript
import { useTimeSeriesData } from "@/lib/useTimeSeriesData";
import { generateSeriesKey } from "@/lib/timeseriesWriter";

// Generate series key
const seriesKey = useMemo(
  () => generateSeriesKey("myMachineName", serial, "measurement"),
  [serial],
);

// Use the hook to read data
const seriesData = useTimeSeriesData({
  seriesKey,
  timeWindowMs: 60 * 60 * 1000, // 1 hour
  refreshIntervalMs: 100, // Refresh every 100ms
});

return {
  seriesData, // { timestamps, values, isLoading, latest, count, range }
  // ...
};
```

### 5. Update Chart Components

**Before:**
```typescript
import { seriesToUPlotData } from "@/lib/timeseries";

const [timestamps, values] = seriesToUPlotData(seriesData.short);
```

**After:**
```typescript
// Data is already in the correct format
const { timestamps, values } = seriesData;
```

## Complete Example: Mock1 Namespace

See [mock1Namespace.ts](../machines/mock/mock1/mock1Namespace.ts) for a complete example showing:
- Writer setup in the message handler
- Writing multiple timeseries fields
- Proper series key generation

## Complete Example: Component

See [useAnalogInputTestMachine.ts](../machines/analoginputtestmachine/useAnalogInputTestMachine.ts) for a complete example showing:
- Using the `useTimeSeriesData` hook
- Proper series key management with `useMemo`
- Returning data in the correct format

## Namespaces That Need Migration

The following namespaces still need to be updated to use IndexedDB:

1. ✅ `mock1Namespace.ts` - **DONE**
2. ✅ `useAnalogInputTestMachineNamespace.ts` - **DONE**
3. ❌ `aquapath1Namespace.ts` - TODO
4. ❌ `extruder2Namespace.ts` - TODO
5. ❌ `extruder3Namespace.ts` - TODO
6. ❌ `laser1Namespace.ts` - TODO
7. ❌ `winder2Namespace.ts` - TODO
8. ❌ `buffer1Namespace.ts` - TODO (if has timeseries)

## Important Notes

### Series Key Naming

- Use lowercase for machine type: `mock1`, `extruder`, `laser`
- Include serial number for machine-specific data
- Use descriptive field names: `amplitude1`, `temperature`, `pressure`

### When to Write to IndexedDB

Only write timeseries data (high-frequency measurements) to IndexedDB:
- ✅ Sensor readings (temperature, pressure, flow)
- ✅ Signal amplitudes
- ✅ Real-time measurements
- ❌ State changes (mode transitions)
- ❌ Configuration values
- ❌ Status flags

### Memory Considerations

The old `TimeSeries` objects in the store can be removed once IndexedDB is in use:

**Before:**
```typescript
export type MyNamespaceStore = {
  measurement: TimeSeries;
  temperature: TimeSeries;
  // ...
};
```

**After:**
```typescript
export type MyNamespaceStore = {
  currentMeasurement: number | null;
  currentTemperature: number | null;
  // Only keep latest values, not full history
};
```

## Testing

After migration, verify:

1. Data appears in IndexedDB (Chrome DevTools → Application → IndexedDB)
2. Charts display correctly
3. Live updates work (data refreshes)
4. Historical data is accessible
5. No memory leaks (check browser task manager)

## Rollback

If issues arise, the old `timeseries.ts` system is still available. To rollback:

1. Revert namespace handler signature changes
2. Revert component hooks to use `createTimeSeries`
3. Remove IndexedDB initialization from `App.tsx`

## Questions?

See the [main README](./README-timeseries.md) for more details on the IndexedDB system.
