import { useEffect, useRef, useState } from "react";

export const FPS_60 = 16; // Backend also uses 16ms for 60 FPS not 16.67

export function useThrottle<T>(value: T, delay: number): T {
  const [throttledValue, setThrottledValue] = useState<T>(value);
  const lastCallTimeRef = useRef<number>(0);

  useEffect(() => {
    const now = Date.now();
    if (now - lastCallTimeRef.current >= delay) {
      setThrottledValue(value);
      lastCallTimeRef.current = now;
    }
  }, [value, delay]);

  return throttledValue;
}
