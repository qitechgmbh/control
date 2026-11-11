import React, { createContext, useContext } from "react";
import { MachineIdentificationUnique } from "@/machines/types";

const GraphHistoryContext = createContext<MachineIdentificationUnique | null>(
  null,
);

/**
 * Tells the graphs below which machine to load database history for. Graphs outside
 * a provider (or series without a `history` key) show live data only.
 */
export function GraphHistoryProvider({
  machine,
  children,
}: {
  machine: MachineIdentificationUnique;
  children: React.ReactNode;
}) {
  return (
    <GraphHistoryContext.Provider value={machine}>
      {children}
    </GraphHistoryContext.Provider>
  );
}

export function useGraphHistoryMachine(): MachineIdentificationUnique | null {
  return useContext(GraphHistoryContext);
}
