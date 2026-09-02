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

// Stable fallback returned when useMarkerContext is called outside a provider.
// Defined at module level so setters have stable identities across renders.
const DEFAULT_CONTEXT: MarkerContextType = {
  machineId: null,
  setMachineId: () => {},
  currentTimestamp: null,
  setCurrentTimestamp: () => {},
  currentValue: null,
  setCurrentValue: () => {},
};

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
  // Return stable module-level default when not within a provider so that
  // effects depending on these setters don't re-run on every render.
  if (!context) {
    return DEFAULT_CONTEXT;
  }
  return context;
}
