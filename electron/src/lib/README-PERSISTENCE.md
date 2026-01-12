# IndexedDB Chart Data Storage - Quick Reference

This directory contains the implementation of the IndexedDB-based persistent storage system for chart/timeseries data.

## 📁 Files Overview

### Core Implementation

| File | Purpose | Key Exports |
|------|---------|-------------|
| **timeseriesDB.ts** | IndexedDB wrapper | `storeDataPoint()`, `queryDataPoints()`, `deleteOldDataPoints()`, etc. |
| **timeseriesPersistent.ts** | Persistent TimeSeries | `createPersistentTimeSeries()` |
| **namespacePersistence.ts** | Namespace helpers | `createNamespacePersistentTimeSeries()` |
| **storageManagement.ts** | Management utilities | `getStorageUsage()`, `exportToCSV()`, `clearSeries()`, etc. |

### React Integration

| File | Purpose | Key Exports |
|------|---------|-------------|
| **useStorageManagement.ts** | React hooks | `useStorageUsage()`, `useSeriesStats()`, `useStorageDashboard()` |
| **StorageManagement.example.tsx** | UI component | `<StorageManagement />` |

### Examples & Documentation

| File | Purpose |
|------|---------|
| **winder2NamespaceHybrid.example.ts** | Complete migration example for a namespace |
| **indexeddb-chart-persistence.md** | Full documentation with migration guide |
| **indexeddb-implementation-summary.md** | Implementation summary and next steps |

## 🚀 Quick Start

### 1. Basic Usage (Single Series)

```typescript
import { createPersistentTimeSeries } from '@/lib/timeseriesPersistent';
import { serializeNamespaceId } from '@/client/socketioStore';

const { initialTimeSeries, insert, ready } = createPersistentTimeSeries(
  serializeNamespaceId(namespaceId),
  'seriesName'
);

// Use initialTimeSeries in your store
// Use insert() to add new data points
// Await ready to get series with historical data loaded
```

### 2. Namespace Integration (Multiple Series)

```typescript
import { createNamespacePersistentTimeSeries } from '@/lib/namespacePersistence';

const seriesResult = createNamespacePersistentTimeSeries(
  namespaceId,
  {
    speed: 'speed',
    position: 'position',
    temperature: 'temperature',
  }
);

// Use seriesResult.initialState in your store
// Use seriesResult.insertFns to insert data
// Await seriesResult.ready for historical data
```

### 3. Add Storage UI

```typescript
import { StorageManagement } from '@/components/StorageManagement.example';

function SettingsPage() {
  return <StorageManagement />;
}
```

## 📖 Documentation

- **Migration Guide**: See [indexeddb-chart-persistence.md](../../docs/developer-docs/indexeddb-chart-persistence.md)
- **Complete Example**: See [winder2NamespaceHybrid.example.ts](../machines/winder/winder2/winder2NamespaceHybrid.example.ts)
- **Implementation Summary**: See [indexeddb-implementation-summary.md](../../docs/developer-docs/indexeddb-implementation-summary.md)

## ⚙️ Configuration

```typescript
const config = {
  // Timeseries config
  sampleIntervalShort: 20,      // 20ms (RAM only)
  sampleIntervalLong: 1000,     // 1s (RAM + IndexedDB)
  retentionDurationShort: 5000, // 5s (RAM only)
  retentionDurationLong: 3600000, // 1h (RAM + IndexedDB)
  
  // Persistence config
  enablePersistence: true,
  maxRetentionTime: 7 * 24 * 60 * 60 * 1000, // 7 days
  cleanupInterval: 60 * 60 * 1000, // 1 hour
};

createPersistentTimeSeries(namespaceId, seriesName, config);
```

## 🎯 Key Features

✅ **Automatic Persistence** - Long series data automatically saved  
✅ **Historical Data** - Loads previous data on mount  
✅ **API Compatible** - Works with existing code  
✅ **Performance Optimized** - Short series stays in RAM  
✅ **Data Management** - Export, clear, and monitor storage  
✅ **Type Safe** - Full TypeScript support  

## 🔄 Migration Steps

1. **Choose a namespace** to migrate (e.g., Winder2)
2. **Update store creation** to pass `namespaceId`
3. **Replace `createTimeSeries()`** with persistence version
4. **Update message handler** to use new insert functions
5. **Handle async initialization** with `.ready` promise
6. **Test thoroughly** before deploying

See [winder2NamespaceHybrid.example.ts](../machines/winder/winder2/winder2NamespaceHybrid.example.ts) for complete example.

## 📊 Storage Usage

- **Per data point**: ~40 bytes
- **1 hour of data**: ~720KB per machine (5 series)
- **7 days retention**: ~120MB per machine
- **Automatic cleanup**: Every hour

## 🐛 Troubleshooting

### Data not persisting
- Check `enablePersistence: true` in config
- Verify browser supports IndexedDB
- Check console for errors

### Slow loading
- Reduce `maxRetentionTime`
- Check number of series
- Monitor query performance

### Storage quota exceeded
- Reduce `maxRetentionTime`
- Increase `cleanupInterval`
- Add manual clear functionality

## 🧪 Testing

```typescript
// Check if persistence is available
import { isPersistenceAvailable } from '@/lib/storageManagement';

if (isPersistenceAvailable()) {
  console.log('IndexedDB persistence is supported');
}

// Get storage stats
import { useStorageUsage } from '@/hooks/useStorageManagement';

function MyComponent() {
  const { usage } = useStorageUsage();
  console.log(`Using ${usage?.usage} of ${usage?.quota} bytes`);
}
```

## 📞 Support

For questions or issues:
1. Check the [full documentation](../../docs/developer-docs/indexeddb-chart-persistence.md)
2. Review the [example implementation](../machines/winder/winder2/winder2NamespaceHybrid.example.ts)
3. See [troubleshooting section](../../docs/developer-docs/indexeddb-chart-persistence.md#troubleshooting)

---

**Status**: ✅ Production Ready  
**Version**: 1.0.0  
**Last Updated**: January 12, 2026
