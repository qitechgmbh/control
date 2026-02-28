import React, { createContext, useContext, useState } from "react";

type MarkerContextType = {
  machineId: string | null;
  setMachineId: (id: string) => void;
  currentTimestamp: number | null;
  setCurrentTimestamp: (timestamp: number) => void;
  currentValue: number | null;
  setCurrentValue: (value: number | null) => void;
};

const MarkerContext = createContext<MarkerContextType | null>(null);

export function MarkerProvider({ children }: { children: React.ReactNode }) {
  const [machineId, setMachineId] = useState<string | null>(null);
  const [currentTimestamp, setCurrentTimestamp] = useState<number | null>(null);
  const [currentValue, setCurrentValue] = useState<number | null>(null);

  return (
    <MarkerContext.Provider
      value={{
        machineId,
        setMachineId,
        currentTimestamp,
        setCurrentTimestamp,
        currentValue,
        setCurrentValue,
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
      currentValue: null,
      setCurrentValue: () => {},
    };
  }
  return context;
}
