import { mainNamespaceStore, mainMessageHandler } from "./mainNamespace";

// Soft reset function
export function softReloadMainNamespaceStore() {
  // 1️⃣ Reset the store to initial state
  mainNamespaceStore.setState({
    ethercatDevices: null,
    machines: null,
    ethercatInterfaceDiscovery: null,
  });

  // 2️⃣ Re-attach the message handler if needed
  // This ensures future socket events still update the store
  mainMessageHandler(mainNamespaceStore);

  console.log("MainNamespaceStore soft reloaded");
}
