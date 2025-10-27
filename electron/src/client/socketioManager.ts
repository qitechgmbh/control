// Singleton manager to eagerly initialize all namespaces
class SocketManager {
  private static instance: SocketManager;
  private store = useSocketioStore.getState();

  private constructor() {}

  static getInstance() {
    if (!SocketManager.instance) {
      SocketManager.instance = new SocketManager();
    }
    return SocketManager.instance;
  }

  // Initialize main namespace immediately
  initMainNamespace<S>(
    createStore: () => StoreApi<S>,
    createEventHandler: (store: StoreApi<S>, updater: ThrottledStoreUpdater<S>) => EventHandler
  ) {
    const mainNamespace: NamespaceId = { type: "main" };
    if (!this.store.hasNamespace(mainNamespace)) {
      this.store.initNamespace(mainNamespace, createStore, createEventHandler);
    }
  }

  // Initialize a machine namespace eagerly
  initMachineNamespace<S>(
    machineId: MachineIdentificationUnique,
    createStore: () => StoreApi<S>,
    createEventHandler: (store: StoreApi<S>, updater: ThrottledStoreUpdater<S>) => EventHandler
  ) {
    const ns: NamespaceId = { type: "machine", machine_identification_unique: machineId };
    if (!this.store.hasNamespace(ns)) {
      this.store.initNamespace(ns, createStore, createEventHandler);
    }
  }

  // Initialize multiple machine namespaces
  initMachineNamespaces<S>(
    machineIds: MachineIdentificationUnique[],
    createStore: () => StoreApi<S>,
    createEventHandler: (store: StoreApi<S>, updater: ThrottledStoreUpdater<S>) => EventHandler
  ) {
    machineIds.forEach((id) => this.initMachineNamespace(id, createStore, createEventHandler));
  }
}

// Then in your hook, we just subscribe (no creation)
export function createNamespaceHookImplementation<S>({
  createStore,
  createEventHandler,
}: NamespaceImplementationConfig<S>): NamespaceImplementationResult<S> {
  return function useNamespace(namespaceId: NamespaceId): S {
    const { getNamespace, hasNamespace } = useSocketioStore();

    // no init here, manager handles it
    const initialState = useMemo(() => createStore().getState(), [createStore]);
    const store = useSyncExternalStore(
      (callback) => {
        if (hasNamespace(namespaceId)) {
          return getNamespace(namespaceId)?.store.subscribe(callback) || (() => {});
        }
        return () => {};
      },
      () => {
        if (hasNamespace(namespaceId)) {
          return (getNamespace(namespaceId)?.store.getState() as S) || initialState;
        }
        return initialState;
      },
    );

    return store;
  };
}