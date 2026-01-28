import React, { createContext, useContext, useState } from "react";

type MarkerContextType = {
  machineId: string | null;
  setMachineId: (id: string) => void;
  currentTimestamp: number | null;
  setCurrentTimestamp: (timestamp: number) => void;
};

const MarkerContext = createContext<MarkerContextType | null>(null);

export function MarkerProvider({ children }: { children: React.ReactNode }) {
  const [machineId, setMachineId] = useState<string | null>(null);
  const [currentTimestamp, setCurrentTimestamp] = useState<number | null>(null);

  return (
    <MarkerContext.Provider
      value={{
        machineId,
        setMachineId,
        currentTimestamp,
        setCurrentTimestamp,
      }}
    >
      {children}
    </MarkerContext.Provider>
  );
}

export function useMarkerContext() {
  const context = useContext(MarkerContext);
  // Return a default context if not within a provider (for non-graph pages)
  if (!context) {
    return {
      machineId: null,
      setMachineId: () => {},
      currentTimestamp: null,
      setCurrentTimestamp: () => {},
    };
  }
  return context;
}
