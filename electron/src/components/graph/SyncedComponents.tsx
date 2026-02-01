import React, { useState } from "react";
import { BigGraph } from "./BigGraph";
import { GraphControls, FloatingControlPanel } from "./GraphControls";
import { useGraphSync } from "./useGraphSync";
import { BigGraphProps, PropGraphSync, TimeWindowOption } from "./types";
import { GraphExportData } from "./excelExport";
import { useMarkerManager } from "./useMarkerManager";
import { AddMarkerDialog } from "./AddMarkerDialog";
import { useMarkerContext } from "./MarkerContext";

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

function SyncedFloatingControlPanelInner({
  controlProps,
  timeWindowOptions,
  machineId: machineIdProp,
  ...props
}: {
  controlProps?: ReturnType<typeof useGraphSync>["controlProps"];
  timeWindowOptions?: TimeWindowOption[];
  machineId?: string;
}) {
  const defaultSync = useGraphSync();
  const finalProps = controlProps || defaultSync.controlProps;
  const { machineId: machineIdContext, currentTimestamp } = useMarkerContext();

  // Prefer explicit prop so panel and graphs always use the same store
  const detectedMachineId =
    machineIdProp ?? machineIdContext ?? "default";
  const { addMarker } = useMarkerManager(detectedMachineId);
  const [isMarkerDialogOpen, setIsMarkerDialogOpen] = useState(false);

  // Always use current timestamp from context (live time from graphs) or current time
  // As per requirement: "always use the current time"
  const markerTimestamp = currentTimestamp || Date.now();

  const handleAddMarker = (name: string, timestamp: number, color?: string) => {
    addMarker(name, timestamp, color);
  };

  return (
    <>
      <FloatingControlPanel
        {...finalProps}
        timeWindowOptions={timeWindowOptions}
        onAddMarker={() => setIsMarkerDialogOpen(true)}
        {...props}
      />
      <AddMarkerDialog
        open={isMarkerDialogOpen}
        onOpenChange={setIsMarkerDialogOpen}
        onAddMarker={handleAddMarker}
        currentTimestamp={markerTimestamp}
      />
    </>
  );
}

export function SyncedFloatingControlPanel({
  controlProps,
  timeWindowOptions,
  machineId,
  ...props
}: {
  controlProps?: ReturnType<typeof useGraphSync>["controlProps"];
  timeWindowOptions?: TimeWindowOption[];
  machineId?: string;
}) {
  return (
    <SyncedFloatingControlPanelInner
      controlProps={controlProps}
      timeWindowOptions={timeWindowOptions}
      machineId={machineId}
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
