import { useRouterState } from "@tanstack/react-router";

export function useOnSubpath(path: string) {
  const { location } = useRouterState();
  // chek if the current path is a subpath of the given path
  const onSubpath = location.pathname.startsWith(path);
  return onSubpath;
}
