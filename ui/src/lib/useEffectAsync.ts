import { useEffect } from "react";

export function useEffectAsync(
  effect: () => Promise<void>,
  deps: React.DependencyList,
): void {
  useEffect(() => {
    let isMounted = true;

    const executeEffect = async () => {
      if (isMounted) {
        await effect();
      }
    };

    executeEffect();

    return () => {
      isMounted = false;
    };
  }, deps);
}
