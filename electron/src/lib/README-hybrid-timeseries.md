# IndexedDB Timeseries - Backward Compatible API

## Overview

This system provides a **backward-compatible API** that maintains the same interface as the old RAM-based timeseries system while using IndexedDB under the hood. This means **minimal code changes** are needed when migrating.

## Key Benefits

✅ **Same API** - `createTimeSeries()` and `insert()` work exactly as before  
✅ **Automatic IndexedDB** - Data persisted automatically in background  
✅ **Small code changes** - Usually just change the import and add `seriesKey`  
✅ **Memory efficient** - Only keeps small live buffer in RAM  
✅ **Historical data** - IndexedDB provides full history when needed  

## Quick Migration

### Step 1: Change Import

**Before:**
```typescript
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
```

**After:**
```typescript
import { createTimeSeries, TimeSeries } from "@/lib/timeseriesHybrid";
```

### Step 2: Add Series Key

**Before:**
```typescript
const timeSeries = createTimeSeries({
  sampleIntervalShort: 20,
  sampleIntervalLong: 1000,
  retentionDurationShort: 5000,
  retentionDurationLong: 60 * 60 * 1000,
});
```

**After:**
```typescript
const timeSeries = createTimeSeries({
  seriesKey: `myMachine:${serial}:measurement`,
  liveBufferSize: 250, // optional, default: 250
  retentionDurationShort: 5000, // optional
  retentionDurationLong: 60 * 60 * 1000, // optional
});
```

### Step 3: That's It! 🎉

Everything else works exactly the same:

```typescript
const [seriesData, setSeriesData] = useState(timeSeries.initialTimeSeries);

useEffect(() => {
  const dataPoint = { value, timestamp };
  setSeriesData(timeSeries.insert(seriesData, dataPoint));
}, [measurement]);

// Use data same as before
const [timestamps, values] = seriesToUPlotData(seriesData.short);
```

## Complete Example

```typescript
import { createTimeSeries, TimeSeries, seriesToUPlotData } from "@/lib/timeseriesHybrid";

export function useMySensor(serial: number) {
  const timeSeries = useMemo(
    () =>
      createTimeSeries({
        seriesKey: `mySensor:${serial}:temperature`,
        liveBufferSize: 250,
        retentionDurationShort: 5000,
        retentionDurationLong: 60 * 60 * 1000,
      }),
    [serial],
  );

  const [seriesData, setSeriesData] = useState(timeSeries.initialTimeSeries);

  useEffect(() => {
    if (newMeasurement) {
      const dataPoint = {
        value: newMeasurement.value,
        timestamp: newMeasurement.timestamp,
      };
      setSeriesData(timeSeries.insert(seriesData, dataPoint));
    }
  }, [newMeasurement]);

  return { seriesData };
}

// In chart component
function MyChart({ seriesData }: { seriesData: TimeSeries }) {
  const [timestamps, values] = seriesToUPlotData(seriesData.short);
  return <Chart data={{ timestamps, values }} />;
}
```

## How It Works

### Architecture

```
Component calls insert() → Updates live buffer (RAM)
                        ↓
                   Writes to IndexedDB (async)
                        ↓
                   IndexedDB stores point
                        ↓
              Auto-cleanup removes old data
```

### Memory Usage

- **Live Buffer**: Small in-memory buffer (default 250 points ≈ 5 seconds at 50Hz)
- **IndexedDB**: Full history stored on disk (default 24 hours)
- **RAM Usage**: ~10KB per series (vs 5-10MB with old system)

### Data Flow

1. **Insert**: Adds point to live buffer + writes to IndexedDB async
2. **Extract**: Returns live buffer data (immediate)
3. **Cleanup**: Removes old data from IndexedDB automatically

## API Reference

### `createTimeSeries(config)`

Creates a timeseries instance with IndexedDB backing.

**Config:**
```typescript
{
  seriesKey: string;              // Required: Unique key for this series
  liveBufferSize?: number;        // Optional: Buffer size (default: 250)
  retentionDurationShort?: number; // Optional: Short window ms (default: 5000)
  retentionDurationLong?: number;  // Optional: Long window ms (default: 3600000)
}
```

**Returns:**
```typescript
{
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, value: TimeSeriesValue) => TimeSeries;
}
```

### `TimeSeries` Type

Same structure as before:

```typescript
{
  current: TimeSeriesValue | null;
  short: Series;  // 5-second window
  long: Series;   // 1-hour window
}
```

### Helper Functions

All existing helper functions work the same:

- `seriesToUPlotData(series)` - Convert to chart format
- `extractDataFromSeries(series, timeWindow?)` - Get arrays
- `getSeriesMinMax(series)` - Get min/max
- `getSeriesStats(series)` - Get statistics

## Series Key Format

Use a consistent format: `{machineType}:{serial}:{fieldName}`

Examples:
- `mock1:123:amplitude1`
- `extruder:456:temperature`
- `laser:789:power`

## Migration Checklist

For each machine namespace:

- [ ] Change import from `/timeseries` to `/timeseriesHybrid`
- [ ] Add `seriesKey` to `createTimeSeries()` config
- [ ] Wrap in `useMemo()` if serial number is dynamic
- [ ] Test charts display correctly
- [ ] Verify IndexedDB stores data (Chrome DevTools → Application → IndexedDB)

## Comparison: Old vs New

### Old System (RAM-based)

```typescript
// Stores ALL data in RAM
const timeSeries = createTimeSeries({
  sampleIntervalShort: 20,
  sampleIntervalLong: 1000,
  retentionDurationShort: 5000,
  retentionDurationLong: 60 * 60 * 1000,
});
```

**Memory**: 5-10MB per series for 1 hour  
**Persistence**: Lost on refresh  
**History**: Limited by RAM  

### New System (Hybrid)

```typescript
// Stores buffer in RAM, history in IndexedDB
const timeSeries = createTimeSeries({
  seriesKey: "machine:123:temp",
  liveBufferSize: 250,
  retentionDurationShort: 5000,
  retentionDurationLong: 60 * 60 * 1000,
});
```

**Memory**: ~10KB per series  
**Persistence**: Stored in IndexedDB  
**History**: Up to 24 hours (configurable)  

## FAQ

### Q: Do I need to change my chart components?

**A:** No! Charts use the same data format:

```typescript
const [timestamps, values] = seriesToUPlotData(seriesData.short);
```

### Q: What happens if IndexedDB fails?

**A:** The live buffer still works in RAM. Data won't persist, but charts display normally.

### Q: How do I access historical data beyond the buffer?

**A:** Use the `useTimeSeriesWithHistory` hook (advanced usage):

```typescript
const { timestamps, values, isLoading } = useTimeSeriesWithHistory(
  "machine:123:temp",
  seriesData,
  "long"
);
```

### Q: Can I customize buffer sizes?

**A:** Yes, adjust `liveBufferSize` in the config:

```typescript
createTimeSeries({
  seriesKey: "machine:123:temp",
  liveBufferSize: 500, // 500 points instead of 250
});
```

### Q: Is this slower than the old system?

**A:** No! The live buffer is just as fast. IndexedDB writes happen async in the background.

## Migration Example: Mock1

See [mock1Namespace.ts](../machines/mock/mock1/mock1Namespace.ts) for a complete example showing:

1. Import change
2. Series key generation
3. Same insert pattern
4. Same data extraction

## Cleanup Configuration

Data cleanup happens automatically. Configure in [App.tsx](../App.tsx):

```typescript
startCleanupService({
  retentionPeriodMs: 24 * 60 * 60 * 1000, // 24 hours
  cleanupIntervalMs: 60 * 60 * 1000,      // Run every hour
  enabled: true,
});
```

## Next Steps

1. **Test Migration**: Start with one machine namespace
2. **Verify Data**: Check IndexedDB in Chrome DevTools
3. **Monitor Memory**: Should see significant reduction
4. **Roll Out**: Migrate remaining namespaces

## Support

- **Full API docs**: [timeseriesHybrid.ts](./timeseriesHybrid.ts)
- **Migration guide**: [migration-indexeddb-timeseries.md](../../docs/developer-docs/migration-indexeddb-timeseries.md)
- **Architecture**: [ARCHITECTURE-DIAGRAM.md](./ARCHITECTURE-DIAGRAM.md)
