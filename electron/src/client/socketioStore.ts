import { useEffect, useMemo, useRef } from "react";
import { create, StoreApi } from "zustand";
import { produce } from "immer";
import { io, Socket } from "socket.io-client";
import { useSyncExternalStore } from "react";
import { z } from "zod";
import { rustEnumSchema } from "@/lib/types";
import { toastError, toastZodError } from "@/components/Toast";

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
    content: rustEnumSchema({
      Warning: z.string().optional(),
      Error: z.string().optional(),
      Data: dataSchema.optional(),
    }),
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
  | { type: "machine"; vendor: number; serial: number; machine: number };

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
 * Unknown event error handler
 */
export const handleUnknownEventError = (eventName: string) => {
  toastError(
    `Unknown Event`,
    `Namespace can't find schema for event "${eventName}"`,
  );
  throw new Error(`Unknown Event '${eventName}'`);
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
  /** Number of active subscribers to this namespace */
  count: number;
  socket: Socket;
  /** Callback function handling incoming socket messages for this namespace */
  handler: EventHandler;
  /** Zustand store holding the namespace state */
  store: StoreApi<S>;
};

/**
 * Utility function to serialize NamespaceId to string for use as map keys
 */
export function serializeNamespaceId(namespaceId: NamespaceId): string {
  if (namespaceId.type === "main") {
    return "/main";
  } else if (namespaceId.type === "machine") {
    return `/machine/${namespaceId.vendor}/${namespaceId.serial}/${namespaceId.machine}`;
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
    const serial = parseInt(parts[2]);
    const machine = parseInt(parts[3]);
    if (isNaN(vendor) || isNaN(serial) || isNaN(machine)) {
      throw new Error("Invalid namespaceId");
    }
    return { type: "machine", vendor, serial, machine };
  } else {
    throw new Error("Invalid namespaceId");
  }
}

type SocketioStore = {
  baseUrl: string;
  namespaces: Record<string, Namespace<unknown>>;
  getNamespace: (namespaceId: NamespaceId) => Namespace<unknown> | undefined;
  hasNamespace: (namespaceId: NamespaceId) => boolean;
  initNamespace: (
    namespaceId: NamespaceId,
    store: StoreApi<unknown>,
    handler: EventHandler,
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
  initNamespace: (namespaceId, store, eventHandler) => {
    const namespace_path = serializeNamespaceId(namespaceId);

    // check if the namespace already exists
    if (get().hasNamespace(namespaceId)) {
      throw new Error(`Namespace ${namespace_path} already initialized`);
    }

    // create a new socket
    const socket = io(get().baseUrl + namespace_path, { autoConnect: false });

    // default event handler if none provided
    const handler =
      eventHandler ||
      ((event: Event<any>) => {
        console.log(`Received event from ${namespace_path}:`, event);
      });

    // add handlers
    socket.on("connect", () => {
      console.log(`Connected to ${namespace_path}`);
    });
    socket.on("disconnect", () => {
      console.warn(`Disconnected from ${namespace_path}`);

      // clear the store
      set(
        produce((state: SocketioStore) => {
          state.namespaces[namespace_path].store.setState({});
        }),
      );
    });
    socket.on("event", (event: unknown) => {
      // validate the event
      let event_parsed = eventSchema(z.any()).safeParse(event);
      if (!event_parsed.success) {
        toastZodError(event_parsed.error, "Invalid event");
        return;
      }
      // handle the event
      handler(event_parsed.data);
    });

    // store the namespace
    set(
      produce((state: SocketioStore) => {
        state.namespaces[namespace_path] = {
          count: 0,
          socket,
          handler,
          store,
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

    // increment the count
    set(
      produce((state: SocketioStore) => {
        state.namespaces[namespace_path].count++;
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

          // if the count is zero, disconnect the socket and delete it
          if (state.namespaces[namespace_path].count <= 0) {
            state.namespaces[namespace_path].socket.disconnect();
            delete state.namespaces[namespace_path];
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
        const store = createStore();
        const eventHandler = createEventHandler(store);
        initNamespace(namespaceId, store, eventHandler);
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
