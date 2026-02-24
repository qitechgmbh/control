/**
 * App root — sets up routing and creates the main namespace socket once.
 *
 * The main namespace signal is created here (at app scope) and provided via
 * context so all pages can access it without prop drilling.
 *
 * Machine-specific namespaces are created inside their pages so they
 * connect/disconnect with the page lifecycle automatically.
 *
 * No global store. No Zustand. No Immer. No ThrottledStoreUpdater.
 * State lives in SolidJS signals, reactive updates are handled by batch().
 */

import { Router, Route } from "@solidjs/router";
import { createContext, useContext } from "solid-js";
import type { JSX } from "solid-js";
import Sidebar from "./components/Sidebar";
import MachinesPage from "./pages/MachinesPage";
import TestMachineControlPage from "./pages/TestMachineControlPage";
import Mock1ControlPage from "./pages/Mock1ControlPage";
import Mock1GraphPage from "./pages/Mock1GraphPage";
import { createMainNamespace, type MainState } from "./namespaces/main";
import "./styles.css";

// Context so nested pages can read main state without prop drilling
export const MainStateContext = createContext<() => MainState>(() => ({
  machines: null,
  ethercatDevices: null,
  ethercatInterface: null,
}));

export function useMainState() {
  return useContext(MainStateContext);
}

// Shared layout wrapper passed to Router as `root`
function AppLayout(props: { children?: JSX.Element }) {
  const mainState = useMainState();
  return (
    <div class="app-layout">
      <Sidebar mainState={mainState} />
      <main class="app-content">{props.children}</main>
    </div>
  );
}

export default function App() {
  // One socket for the main namespace, alive for the entire app lifetime.
  // createMainNamespace() calls createNamespace() which calls onCleanup()
  // — so the socket disconnects if App ever unmounts.
  const [mainState] = createMainNamespace();

  return (
    <MainStateContext.Provider value={mainState}>
      <Router root={AppLayout}>
        <Route path="/" component={() => <MachinesPage mainState={mainState} />} />
        <Route path="/machines/testmachine/:serial/control" component={TestMachineControlPage} />
        <Route
          path="/machines/testmachine/:serial"
          component={() => (
            <div class="page">
              <h1>Test Machine</h1>
              <p class="muted">Select Control from the sidebar.</p>
            </div>
          )}
        />
        <Route path="/machines/mock1/:serial/control" component={Mock1ControlPage} />
        <Route path="/machines/mock1/:serial/graph" component={Mock1GraphPage} />
        <Route
          path="/machines/mock1/:serial"
          component={() => (
            <div class="page">
              <h1>Mock Machine</h1>
              <p class="muted">Select Control from the sidebar.</p>
            </div>
          )}
        />
      </Router>
    </MainStateContext.Provider>
  );
}
