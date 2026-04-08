import { useCallback, useRef, useState } from "react";

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

  const valueRealRef = useRef<T | undefined>(undefined);
  valueRealRef.current = valueReal;

  const setOptimistic = useCallback((value: T) => {
    setValueOptimistic(value);
    setIsOptimistic(true);
  }, []);

  const setReal = useCallback((value: T) => {
    setValueReal(value);
    setIsOptimistic(false);
    setIsInitialized(true);
  }, []);

  const resetToReal = useCallback(() => {
    setValueOptimistic(valueRealRef.current);
    setIsOptimistic(false);
  }, []);

  return {
    value: isOptimistic ? valueOptimistic : valueReal,
    setOptimistic,
    setReal,
    resetToReal,
    isOptimistic,
    isInitialized,
  };
}
