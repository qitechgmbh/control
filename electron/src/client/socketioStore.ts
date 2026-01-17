import { useEffect, useMemo } from "react";
import { create, StoreApi } from "zustand";
import { produce } from "immer";
import { io, Socket } from "socket.io-client";
import MsgPackParser from "socket.io-msgpack-parser";
import { useSyncExternalStore } from "react";
import { z } from "zod";
import { toastError, toastZodError } from "@/components/Toast";
import { MachineIdentificationUnique } from "@/machines/types";
import { FPS_30 } from "@/lib/constants";
import { mainNamespaceStore } from "./mainNamespace";

/**
 * Simple buffer-based store updater to limit React re-renders to ~30 FPS
 * Writes events to a plain JS object and syncs to store every 33ms
 */
export class ThrottledStoreUpdater<S> {
  private store: StoreApi<S>;
  private buffer: S;
  private syncTimer: NodeJS.Timeout | null = null;
  private readonly syncDelay = FPS_30; // ~30 FPS (33.33ms)

  constructor(store: StoreApi<S>) {
    this.store = store;
    // Initialize buffer with current store state
    this.buffer = { ...store.getState() };
  }

  /**
   * Update the buffer directly (no store update yet)
   * @param updates Partial state to merge into buffer
   */
  update(updates: Partial<S>): void {
    this.buffer = { ...this.buffer, ...updates };
    this.scheduleSync();
  }

  /**
   * Apply a function to update the buffer
   * @param updater Function that takes current buffer and returns new state
   */
  updateWith(updater: (state: S) => S): void {
    this.buffer = updater(this.buffer);
    this.scheduleSync();
  }

  /**
   * Get current buffer state for immediate reads
   */
  getBufferState(): S {
    return this.buffer;
  }

  /**
   * Schedule a sync to store if not already scheduled
   */
  private scheduleSync(): void {
    if (this.syncTimer === null) {
      this.syncTimer = setTimeout(() => {
        this.syncToStore();
      }, this.syncDelay);
    }
  }

  /**
   * Sync the entire buffer to the store
   */
  private syncToStore(): void {
    try {
      this.store.setState(this.buffer);
      this.syncTimer = null;
    } catch (error) {
      console.error("Error syncing to store:", error);
      this.syncTimer = null;
      // Retry after a short delay
      setTimeout(() => {
        this.scheduleSync();
      }, 100);
    }
  }

  /**
   * Force immediate sync of buffer to store
   */
  forceSync(): void {
    if (this.syncTimer) {
      clearTimeout(this.syncTimer);
      this.syncToStore();
    }
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    if (this.syncTimer) {
      clearTimeout(this.syncTimer);
      this.syncTimer = null;
    }
    // Final sync before destroy
    this.syncToStore();
  }
}

/**
 * Creates a throttled event handler that batches store updates for better performance
 * @param originalHandler The original event handler that processes events
 * @returns A new event handler that batches updates at 60 FPS
 */
export function createThrottledEventHandler<S>(
  store: StoreApi<S>,
  originalHandler: (
    event: Event<any>,
    updateBuffer: (updater: (state: S) => S) => void,
  ) => void,
): EventHandler {
  const throttledUpdater = new ThrottledStoreUpdater(store);

  return (event: Event<any>) => {
    originalHandler(event, (updater) => {
      throttledUpdater.updateWith(updater);
    });
  };
}

/**
 * Generic event schema builder
 * Creates a Zod schema for an Event with strongly typed payload
 *
 * @template T The Zod schema for the event data
 * @param dataSchema Zod schema describing the data structure
 * @returns Zod schema for an Event with the specified data type
 */
export function eventSchema<T extends z.ZodTypeAny>(dataSchema: T) {
  return z.object({
    name: z.string(),
    data: dataSchema,
    ts: z.number().int().positive(),
  });
}

/**
 * Type inference helper for Event schema
 */
export type Event<T extends z.ZodTypeAny> = z.infer<
  ReturnType<typeof eventSchema<T>>
>;

/**
 * Namespace identifiers
 */
export type NamespaceId =
  | { type: "main" }
  | {
      type: "machine";
      machine_identification_unique: MachineIdentificationUnique;
    };

/**
 * Event validation error handler
 */
export const handleEventValidationError = (
  error: z.ZodError,
  eventName: string,
) => {
  toastZodError(error, `Event Validation Error for ${eventName}`);
  throw new Error(`Event validation failed for ${eventName}`);
};

/**
 * Unhandled event error handler
 */
export const handleUnhandledEventError = (eventName: string) => {
  toastError(`Unhandled Event`, `Namespace can't handle event "${eventName}"`);
  throw new Error(`Unhandled Event '${eventName}'`);
};

/**
 * Callback signature for handling socket.io messages
 */
export type EventHandler = (event: Event<any>) => void;

/**
 * Represents a namespace with its own store and subscription management
 * @template S The store state type
 */
type Namespace<S> = {
  /** Number of active subscribers to this room */
  count: number;
  socket: Socket;
  /** Callback function handling incoming socket messages for this room */
  handler: EventHandler;
  /** Zustand store holding the room state */
  store: StoreApi<S>;
  /** Timeout ID for disconnection */
  disconnectTimeoutId?: NodeJS.Timeout;
  /** Throttled store updater for batching state updates */
  throttledUpdater: ThrottledStoreUpdater<S>;
};

/**
 * Utility function to serialize NamespaceId to string for use as map keys
 */
export function serializeNamespaceId(namespaceId: NamespaceId): string {
  if (namespaceId.type === "main") {
    return "/main";
  } else if (namespaceId.type === "machine") {
    return `/machine/${namespaceId.machine_identification_unique.machine_identification.vendor}/${namespaceId.machine_identification_unique.machine_identification.machine}/${namespaceId.machine_identification_unique.serial}`;
  } else {
    throw new Error("Invalid namespaceId");
  }
}

/**
 * Utility function to deserialize string back to NamespaceId
 */
export function deserializeNamespaceId(namespaceId: string): NamespaceId {
  const parts = namespaceId.split("/");
  if (parts.length === 2 && parts[0] === "") {
    // /main
    return { type: "main" };
  } else if (parts.length === 5 && parts[0] === "machine") {
    // /machine/0/0/0
    const vendor = parseInt(parts[1]);
    const machine = parseInt(parts[2]);
    const serial = parseInt(parts[3]);
    if (isNaN(vendor) || isNaN(serial) || isNaN(machine)) {
      throw new Error("Invalid namespaceId");
    }
    return {
      type: "machine",
      machine_identification_unique: {
        machine_identification: {
          vendor,
          machine,
        },
        serial,
      },
    };
  } else {
    throw new Error("Invalid namespaceId");
  }
}

type SocketioStore = {
  baseUrl: string;
  namespaces: Record<string, Namespace<unknown>>;
  getNamespace: (namespaceId: NamespaceId) => Namespace<unknown> | undefined;
  hasNamespace: (namespaceId: NamespaceId) => boolean;
  initNamespace: <S>(
    namespaceId: NamespaceId,
    createStore: () => StoreApi<S>,
    createEventHandler: (
      store: StoreApi<S>,
      throttledUpdater: ThrottledStoreUpdater<S>,
    ) => EventHandler,
  ) => void;
  incrementNamespace: (namespaceId: NamespaceId) => void;
  decrementNamespace: (namespaceId: NamespaceId) => void;
};

/**
 * Global socket store singleton that manages socket.io connections and namespaces
 */
const useSocketioStore = create<SocketioStore>()((set, get) => ({
  baseUrl: "http://localhost:3001",
  namespaces: {},
  getNamespace: (namespaceId: NamespaceId) => {
    const namespace_path = serializeNamespaceId(namespaceId);
    return get().namespaces[namespace_path];
  },
  hasNamespace: (namespaceId: NamespaceId) => {
    const namespace_path = serializeNamespaceId(namespaceId);
    return !!get().namespaces[namespace_path];
  },
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  initNamespace: <S>(namespaceId, createStore, createEventHandler) => {
    const namespace_path = serializeNamespaceId(namespaceId);

    // check if the namespace already exists
    if (get().hasNamespace(namespaceId)) {
      throw new Error(`Namespace ${namespace_path} already initialized`);
    }

    // create a new socket
    const socket = io(get().baseUrl + namespace_path, {
      autoConnect: false,
      parser: MsgPackParser,
    });

    // create function to reset the store
    // creating a new store and a new event handler
    const resetStore = (set) => {
      set(
        produce((state: SocketioStore) => {
          const store = createStore();
          const throttledUpdater = new ThrottledStoreUpdater(store);
          const eventHandler = createEventHandler(store, throttledUpdater);
          state.namespaces[namespace_path].store = store;
          state.namespaces[namespace_path].handler = eventHandler;
          state.namespaces[namespace_path].throttledUpdater = throttledUpdater;
        }),
      );
    };

    // add handlers
    socket.on("connect", () => {
      console.log(`Connected to ${namespace_path}`);
      resetStore(set);
    });
    socket.on("disconnect", (reason) => {
      socket.disconnect();
      resetStore(set);
      // Its hacky but i do not care, electron + react is annoying ...
      window.location.reload();
    });

    socket.on("event", (event: unknown) => {
      // validate the event
      const event_parsed = eventSchema(z.any()).safeParse(event);
      if (!event_parsed.success) {
        toastZodError(event_parsed.error, "Invalid event");
        return;
      } // handle the event
      get().namespaces[namespace_path].handler(event_parsed.data);
    });
    // store the namespace initally
    set(
      produce((state: SocketioStore) => {
        const store = createStore();
        const throttledUpdater = new ThrottledStoreUpdater(store);
        const handler = createEventHandler(store, throttledUpdater);
        state.namespaces[namespace_path] = {
          count: 0,
          socket,
          handler,
          store,
          throttledUpdater,
          disconnectTimeoutId: undefined,
        };
      }),
    );

    if (namespace_path !== "/main") {
      const intervalId = setInterval(() => {
        const mainState = mainNamespaceStore.getState();
        const machineExists =
          namespaceId.type === "machine" &&
          mainState.machines?.data?.machines.some(
            (m) =>
              m.machine_identification_unique.serial ===
              namespaceId.machine_identification_unique.serial,
          );
        //console.log(machineExists);
        if (machineExists && !socket.connected) {
          socket.connect();
          clearInterval(intervalId); // stop polling
        }
      }, 500);
    } else {
      socket.connect();
    }
  },
  incrementNamespace: (namespaceId: NamespaceId) => {
    const namespace_path = serializeNamespaceId(namespaceId);

    // check if the namespace exists
    if (!get().hasNamespace(namespaceId)) {
      throw new Error(`Namespace ${namespace_path} not initialized`);
    }

    // increment the count and clear any pending disconnect timeout
    set(
      produce((state: SocketioStore) => {
        state.namespaces[namespace_path].count++;

        // Clear any pending disconnect timeout since we have active subscribers
        if (state.namespaces[namespace_path].disconnectTimeoutId) {
          clearTimeout(state.namespaces[namespace_path].disconnectTimeoutId);
          state.namespaces[namespace_path].disconnectTimeoutId = undefined;
        }
      }),
    );
  },

  decrementNamespace: (namespaceId: NamespaceId) => {
    /*const namespace_path = serializeNamespaceId(namespaceId);

    // check if the namespace exists
    const namespace = get().namespaces[namespace_path];
    if (namespace) {
      set(
        produce((state: SocketioStore) => {
          // decrement the count
          state.namespaces[namespace_path].count--;

          // if the count is zero and it's not the main namespace,
          // set a timeout to check again after 10 seconds
          if (
            namespaceId.type !== "main" &&
            state.namespaces[namespace_path].count <= 0
          ) {
            // Clear any existing timeout first
            if (state.namespaces[namespace_path].disconnectTimeoutId) {
              clearTimeout(
                state.namespaces[namespace_path].disconnectTimeoutId,
              );
            }

            // Create a timeout to check if the namespace is still unused after 1 hour
            const timeoutId = setTimeout(
              () => {
                set(
                  produce((state: SocketioStore) => {
                    const ns = state.namespaces[namespace_path];
                    if (ns && ns.count <= 0) {
                      ns.socket.disconnect();
                      ns.throttledUpdater.destroy(); // Clean up throttled updater
                      delete state.namespaces[namespace_path];
                      console.log(
                        `Namespace ${namespace_path} disconnected after 1h of inactivity`,
                      );
                    }
                  }),
                );
              },
              // 1h until disconnect
              60 * 60 * 1000,
            );

            state.namespaces[namespace_path].disconnectTimeoutId = timeoutId;
          }
        }),
      );
    }*/
  },
}));

/**
 * Configuration for creating a namespace implementation
 * @template S Store state type
 */
export interface NamespaceImplementationConfig<S> {
  /**
   * Function to create the store for this namespace
   * @returns A new Zustand store instance
   */
  createStore: () => StoreApi<S>;

  /**
   * Function that creates a message handler for this namespace
   * @param store The store that will be updated by the handler
   * @param throttledUpdater Throttled updater for batching updates
   * @returns A message handler function
   */
  createEventHandler: (
    store: StoreApi<S>,
    throttledUpdater: ThrottledStoreUpdater<S>,
  ) => EventHandler;
}

export type NamespaceImplementationResult<S> = (namespaceId: NamespaceId) => S;

export function createNamespaceHookImplementation<S>({
  createStore,
  createEventHandler,
}: NamespaceImplementationConfig<S>): NamespaceImplementationResult<S> {
  return function useNamespace(namespaceId: NamespaceId): S {
    // use socketio store
    const {
      incrementNamespace,
      decrementNamespace,
      hasNamespace,
      initNamespace,
      getNamespace,
    } = useSocketioStore();

    // namespace initialization/incrementation/decrementation
    useEffect(() => {
      if (!hasNamespace(namespaceId)) {
        initNamespace(namespaceId, createStore, createEventHandler);
      } else {
        incrementNamespace(namespaceId);
      }
      return () => {
        decrementNamespace(namespaceId);
      };
    }, [namespaceId]);

    // sync store
    const initalState = useMemo(() => createStore().getState(), [createStore]);
    const store = useSyncExternalStore(
      (callback) => {
        if (hasNamespace(namespaceId)) {
          return (
            getNamespace(namespaceId)?.store.subscribe(callback) || (() => {})
          );
        }
        return () => {};
      },
      () => {
        if (hasNamespace(namespaceId)) {
          return (
            (getNamespace(namespaceId)?.store.getState() as S) || initalState
          );
        }
        return initalState;
      },
    );

    return store;
  };
}
