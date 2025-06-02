import { useEffect, useMemo } from "react";
import { create, StoreApi } from "zustand";
import { produce } from "immer";
import { io, Socket } from "socket.io-client";
import MsgPackParser from "socket.io-msgpack-parser";
import { useSyncExternalStore } from "react";
import { z } from "zod";
import { toastError, toastZodError } from "@/components/Toast";
import { MachineIdentificationUnique } from "@/machines/types";

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
    createEventHandler: (store: StoreApi<S>) => EventHandler,
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
          const eventHandler = createEventHandler(store);
          state.namespaces[namespace_path].store = store;
          state.namespaces[namespace_path].handler = eventHandler;
        }),
      );
    };

    // add handlers
    socket.on("connect", () => {
      console.log(`Connected to ${namespace_path}`);

      // reset the store
      resetStore(set);
    });
    socket.on("disconnect", (reason) => {
      socket.disconnect();
      console.warn(`Disconnected from ${namespace_path}, reason: ${reason}`);

      // reset the store
      resetStore(set);

      // Attempt to reconnect after a short delay for any disconnect reason
      console.log(
        `Disconnected from ${namespace_path}, attempting reconnect in 1s...`,
      );
      setTimeout(() => {
        if (get().hasNamespace(namespaceId) && !socket.connected) {
          console.log(`Reconnecting to ${namespace_path}...`);
          socket.connect();
        }
      }, 1000);
    });

    socket.on("event", (event: unknown) => {
      // validate the event
      const event_parsed = eventSchema(z.any()).safeParse(event);
      if (!event_parsed.success) {
        toastZodError(event_parsed.error, "Invalid event");
        return;
      }
      // handle the event
      get().namespaces[namespace_path].handler(event_parsed.data);
    });

    // store the namespace initally
    set(
      produce((state: SocketioStore) => {
        const store = createStore();
        const handler = createEventHandler(store);
        state.namespaces[namespace_path] = {
          count: 0,
          socket,
          handler,
          store,
          disconnectTimeoutId: undefined,
        };
      }),
    );

    // finally connect
    socket.connect();
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
    const namespace_path = serializeNamespaceId(namespaceId);

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

            // Create a timeout to check if the namespace is still unused after 10 seconds
            const timeoutId = setTimeout(
              () => {
                set(
                  produce((state: SocketioStore) => {
                    const ns = state.namespaces[namespace_path];
                    if (ns && ns.count <= 0) {
                      ns.socket.disconnect();
                      delete state.namespaces[namespace_path];
                      console.log(
                        `Namespace ${namespace_path} disconnected after 10s of inactivity`,
                      );
                    }
                  }),
                );
              },
              // 1 year in milliseconds
              // this is a workaround to avoid the current issues with the event reloading
              // perf improvements are tracked by #269
              365 * 24 * 60 * 60 * 1000,
            );

            state.namespaces[namespace_path].disconnectTimeoutId = timeoutId;
          }
        }),
      );
    }
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
   * @returns A message handler function
   */
  createEventHandler: (store: StoreApi<S>) => EventHandler;
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
