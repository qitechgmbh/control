import React from "react";
import { BigGraph } from "./BigGraph";
import { GraphControls, FloatingControlPanel } from "./GraphControls";
import { useGraphSync } from "./useGraphSync";
import { BigGraphProps, PropGraphSync, TimeWindowOption } from "./types";
import { GraphExportData } from "./excelExport";

export function SyncedBigGraph({
  syncGraph: externalSyncGraph,
  registerForExport,
  unregisterFromExport,
  ...props
}: Omit<BigGraphProps, "syncGraph"> & {
  syncGraph?: PropGraphSync;
  registerForExport?: (
    graphId: string,
    getDataFn: () => GraphExportData | null,
  ) => void;
  unregisterFromExport?: (graphId: string) => void;
}) {
  const defaultSync = useGraphSync();

  return (
    <BigGraph
      {...props}
      syncGraph={externalSyncGraph || defaultSync.syncGraph}
      onRegisterForExport={registerForExport}
      onUnregisterFromExport={unregisterFromExport}
    />
  );
}

export function SyncedGraphControls({
  controlProps,
  timeWindowOptions,
  ...props
}: {
  controlProps?: ReturnType<typeof useGraphSync>["controlProps"];
  timeWindowOptions?: TimeWindowOption[];
}) {
  const defaultSync = useGraphSync();
  const finalProps = controlProps || defaultSync.controlProps;

  return (
    <GraphControls
      {...finalProps}
      timeWindowOptions={timeWindowOptions}
      {...props}
    />
  );
}

export function SyncedFloatingControlPanel({
  controlProps,
  timeWindowOptions,
  ...props
}: {
  controlProps?: ReturnType<typeof useGraphSync>["controlProps"];
  timeWindowOptions?: TimeWindowOption[];
}) {
  const defaultSync = useGraphSync();
  const finalProps = controlProps || defaultSync.controlProps;

  return (
    <FloatingControlPanel
      {...finalProps}
      timeWindowOptions={timeWindowOptions}
      {...props}
    />
  );
}

export function AutoSyncedBigGraph({
  syncHook,
  ...props
}: Omit<BigGraphProps, "syncGraph"> & {
  syncHook: ReturnType<typeof useGraphSync>;
}) {
  return (
    <SyncedBigGraph
      {...props}
      syncGraph={syncHook.syncGraph}
      registerForExport={syncHook.registerGraphForExport}
      unregisterFromExport={syncHook.unregisterGraphFromExport}
    />
  );
}
