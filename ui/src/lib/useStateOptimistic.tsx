import { useState } from "react";

export function useStateOptimistic<T>(): {
  value: T | undefined;
  setOptimistic: (value: T) => void;
  setReal: (value: T) => void;
  resetToReal: () => void;
  isOptimistic: boolean;
  isInitialized: boolean;
} {
  const [valueOptimistic, setValueOptimistic] = useState<T | undefined>(
    undefined,
  );
  const [valueReal, setValueReal] = useState<T | undefined>(undefined);
  const [isOptimistic, setIsOptimistic] = useState<boolean>(false);
  const [isInitialized, setIsInitialized] = useState<boolean>(false);

  return {
    value: isOptimistic ? valueOptimistic : valueReal,
    setOptimistic: (value: T) => {
      setValueOptimistic(value);
      setIsOptimistic(true);
    },
    setReal: (value: T) => {
      setValueReal(value);
      setIsOptimistic(false);
      setIsInitialized(true);
    },
    resetToReal: () => {
      setValueOptimistic(valueReal);
      setIsOptimistic(false);
    },
    isOptimistic,
    isInitialized,
  };
}
