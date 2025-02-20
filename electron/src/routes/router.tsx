import { createMemoryHistory, createRouter } from "@tanstack/react-router";
import { rootTree } from "./routes";

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const history = createMemoryHistory({
  initialEntries: ["/_sidebar/machines/123/winder1/general"],
});
export const router = createRouter({ routeTree: rootTree, history: history });
