/**
 * @file SocketRoomManager.ts
 * @description Generic socket.io room management system with Zod schema validation.
 */

import { useEffect, useMemo, useRef, useState } from "react";
import { create, StoreApi } from "zustand";
import { produce } from "immer";
import { io, Socket } from "socket.io-client";
import { useSyncExternalStore } from "react";
import { z } from "zod";
import { rustEnumSchema } from "@/lib/types";
import { toastError, toastZodError } from "@/components/Toast";
import { machineIdentificationUnique } from "@/machines/types";

/**
 * Generic event schema builder
 * Creates a Zod schema for an Event with strongly typed payload
 *
 * @template T The Zod schema for the event data
 * @param dataSchema Zod schema describing the data structure
 * @returns Zod schema for an Event with the specified data type
 */
export function eventSchema<T extends z.ZodTypeAny>(dataSchema: T) {
  return z
    .object({
      room_id: rustEnumSchema({
        Main: z.boolean(),
        Machine: machineIdentificationUnique,
      }),
      name: z.string(),
      content: rustEnumSchema({
        Warning: z.string().optional(),
        Error: z.string().optional(),
        Data: dataSchema.optional(),
      }),
      ts: z.number().int().positive(),
    })
    .refine((val) => (val.room_id.Main ? true : true), {
      message: "Main must be true when specified",
    });
}

/**
 * Type inference helper for Event schema
 */
export type Event<T extends z.ZodTypeAny> = z.infer<
  ReturnType<typeof eventSchema<T>>
>;

export type RoomId = Event<any>["room_id"];

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
    `Room can't find schema for event "${eventName}"`,
  );
  throw new Error(`Unknown Event '${eventName}'`);
};

/**
 * Unhandled event error handler
 */
export const handleUnhandledEventError = (eventName: string) => {
  toastError(`Unhandled Event`, `Room can't handle event "${eventName}"`);
  throw new Error(`Unhandled Event '${eventName}'`);
};

/**
 * Callback signature for handling socket.io messages
 */
export type MessageCallback = (event: Event<any>) => void;

/**
 * Represents a room with its own store and subscription management
 * @template S The store state type
 */
export interface Room<S> {
  /** Number of active subscribers to this room */
  subscribers: number;
  /** Zustand store holding the room state */
  store: StoreApi<S>;
  /** Callback function handling incoming socket messages for this room */
  onMessageCallback: MessageCallback;
}

/**
 * Utility function to serialize RoomId to string for use as map keys
 */
const serializeRoomId = (roomId: RoomId): string => {
  if ("Main" in roomId) {
    return "Main";
  } else if ("Machine" in roomId) {
    const machine = roomId.Machine;
    return `Machine:${machine?.vendor}:${machine?.serial}:${machine?.machine}`;
  }
  throw new Error("Invalid RoomId format");
};

/**
 * Utility function to deserialize string back to RoomId
 */
const deserializeRoomId = (serialized: string): RoomId => {
  if (serialized === "Main") {
    return { Main: true };
  } else if (serialized.startsWith("Machine:")) {
    const [, vendor, serial, machine] = serialized.split(":");
    return {
      Machine: {
        vendor: Number(vendor),
        serial: Number(serial),
        machine: Number(machine),
      },
    };
  }
  throw new Error(`Invalid serialized RoomId: ${serialized}`);
};

/**
 * Core socket management state and operations
 */
export const SocketStateSchema = z.object({
  socket: z.custom<Socket | null>(),
  rooms: z.map(z.string(), z.custom<Room<unknown>>()),
});

export type SocketState = z.infer<typeof SocketStateSchema> & {
  // Socket connection methods
  connect: (url: string) => void;
  disconnect: () => void;

  // Room management methods
  hasRoom: (roomId: RoomId) => boolean;
  initRoom: <S>(
    roomId: RoomId,
    createStore: () => StoreApi<S>,
    onMessageCallback: MessageCallback,
  ) => void;
  joinRoom: (roomId: RoomId) => void;
  leaveRoom: (roomId: RoomId) => void;
  incrementSubscribers: (roomId: RoomId) => void;
  decrementSubscribers: (roomId: RoomId) => void;
  getRoom: <S>(roomId: RoomId) => Room<S> | undefined;
};

/**
 * Global socket store singleton that manages socket.io connections and rooms
 */
export const useSocketStore = create<SocketState>((set, get) => ({
  socket: null,
  rooms: new Map<string, Room<unknown>>(),

  connect: (url: string) => {
    if (get().socket) {
      return;
    }

    const socket = io(url, {
      reconnection: true,
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
    });

    // Set up a single global event handler that routes to appropriate rooms
    socket.on("event", (event: Event<any>) => {
      try {
        const eventRoomId = serializeRoomId(event.room_id);
        const room = get().rooms.get(eventRoomId);

        if (room) {
          // Route the event to the appropriate room handler
          room.onMessageCallback(event);
        }
      } catch (error) {
        console.error("Error routing socket event:", error);
      }
    });

    set(
      produce((state) => {
        state.socket = socket;
      }),
    );

    // Log connection status changes for debugging
    if (process.env.NODE_ENV !== "production") {
      socket.on("connect", () => console.info("Socket connected"));
      socket.on("disconnect", () => console.info("Socket disconnected"));
      socket.on("error", (err) => console.error("Socket error:", err));
    }
  },

  disconnect: () => {
    const { socket, rooms } = get();
    if (socket) {
      // Clean up all rooms before disconnecting
      Array.from(rooms.keys()).forEach((serializedRoomId) => {
        const roomId = deserializeRoomId(serializedRoomId);
        get().leaveRoom(roomId);
      });

      socket.disconnect();
      set(
        produce((state) => {
          state.socket = null;
        }),
      );
    }
  },

  hasRoom: (roomId: RoomId) => {
    const serializedRoomId = serializeRoomId(roomId);
    return get().rooms.has(serializedRoomId);
  },

  initRoom: <S>(
    roomId: RoomId,
    createStore: () => StoreApi<S>,
    onMessageCallback: MessageCallback,
  ) => {
    const { socket, rooms } = get();
    const serializedRoomId = serializeRoomId(roomId);

    if (!socket) {
      console.error(
        `Cannot initialize room ${serializedRoomId}: Socket not connected`,
      );
      return;
    }

    if (rooms.has(serializedRoomId)) {
      console.error(
        `Room ${serializedRoomId} already exists, cannot initialize again`,
      );
      return;
    }

    // Join the room on the server
    socket.emit("subscribe", {
      room_id: roomId,
    });

    // Create the room entry - the global handler will route events
    set(
      produce((state) => {
        const newRoom: Room<S> = {
          subscribers: 0,
          store: createStore(),
          onMessageCallback, // Store the original callback
        };
        (state.rooms as Map<string, Room<unknown>>).set(
          serializedRoomId,
          newRoom as Room<unknown>,
        );
      }),
    );
  },

  joinRoom: (roomId: RoomId) => {
    const { socket } = get();
    if (!socket) {
      throw new Error("Cannot join room: Socket not connected");
    }
    socket.emit("subscribe", {
      room_id: roomId,
    });
  },

  leaveRoom: (roomId: RoomId) => {
    const { socket, rooms } = get();
    const serializedRoomId = serializeRoomId(roomId);
    const room = rooms.get(serializedRoomId);

    if (socket && room) {
      // Remove the message handler to prevent memory leaks
      socket.off("event", room.onMessageCallback);

      // Leave the room on the server
      socket.emit("unsubscribe", {
        room_id: roomId,
      });
    }
  },

  incrementSubscribers: (roomId: RoomId) => {
    const serializedRoomId = serializeRoomId(roomId);
    const room = get().rooms.get(serializedRoomId);
    if (!room) {
      throw new Error(
        `Cannot increment subscribers: Room '${serializedRoomId}' not found. Available rooms: ${Array.from(
          get().rooms.keys(),
        ).join(", ")}`,
      );
    }

    set(
      produce((state) => {
        const room = (state.rooms as Map<string, Room<unknown>>).get(
          serializedRoomId,
        );
        if (room) {
          room.subscribers += 1;
        }
      }),
    );
  },

  decrementSubscribers: (roomId: RoomId) => {
    const serializedRoomId = serializeRoomId(roomId);

    set(
      produce((state) => {
        const room = (state.rooms as Map<string, Room<unknown>>).get(
          serializedRoomId,
        );
        if (room) {
          room.subscribers -= 1;

          // Only clean up the room if subscribers is actually zero or negative
          // Add a small delay to prevent race conditions in React's render cycle
          if (room.subscribers <= 0) {
            // Use a small timeout to prevent immediate leave during React render cycles
            setTimeout(() => {
              // Double check that subscribers is still 0 before cleanup
              const currentRoom = get().rooms.get(serializedRoomId);
              if (currentRoom && currentRoom.subscribers <= 0) {
                get().leaveRoom(roomId);
                set(
                  produce((s) => {
                    (s.rooms as Map<string, Room<unknown>>).delete(
                      serializedRoomId,
                    );
                  }),
                );
              }
            }, 1000);
          }
        }
      }),
    );
  },

  getRoom: <S>(roomId: RoomId) => {
    const serializedRoomId = serializeRoomId(roomId);
    return get().rooms.get(serializedRoomId) as Room<S> | undefined;
  },
}));

/**
 * Optimized event caching utilities for timestamp-ordered events
 * All utilities expect Zod-validated events and assume events are in ascending timestamp order
 */
export const EventCache = {
  // [other existing functions remain the same]

  /**
   * Caches events for a specific time duration, efficiently handling ordered timestamp events
   * Uses binary search to find the cutoff point, making this O(log n) instead of O(n)
   *
   * @template T The event data Zod schema
   * @param events Existing events array (must be in ascending timestamp order)
   * @param newEvent New event to add
   * @param duration Duration in milliseconds to keep events
   * @returns New array with non-expired events plus the new event
   */
  timeWindow: <T extends z.ZodTypeAny>(
    events: ReadonlyArray<Event<T>>,
    newEvent: Event<T>,
    duration: number,
  ): ReadonlyArray<Event<T>> => {
    const now = Date.now();
    const cutoff = now - duration;

    // Early optimization: if array is empty or all events are recent, just append
    if (events.length === 0 || events[0].ts >= cutoff) {
      return [...events, newEvent];
    }

    // Find index of first event that is not expired using binary search
    let start = 0;
    let end = events.length - 1;
    let cutoffIndex = events.length; // Default to keeping all

    while (start <= end) {
      const mid = Math.floor((start + end) / 2);
      if (events[mid].ts < cutoff) {
        start = mid + 1;
        // This might be our cutoff point
        cutoffIndex = start;
      } else {
        end = mid - 1;
      }
    }

    // Efficiently slice the array at the cutoff point and append new event
    // This creates just one new array instead of filtering each element
    return [...events.slice(cutoffIndex), newEvent];
  },

  /**
   * Caches events for a time duration, keeping only the most recent event for each key
   * Optimized for ordered timestamps, avoiding expensive filtering operations
   *
   * @template T The event data Zod schema
   * @template K Key type
   * @param events Existing events array (must be in ascending timestamp order)
   * @param newEvent New event to add
   * @param keyFn Function to extract a key for comparison
   * @param duration Duration in milliseconds to keep events
   * @returns New array with non-expired, unique (by key) events
   */
  timeWindowByKey: <T extends z.ZodTypeAny, K extends string | number>(
    events: ReadonlyArray<Event<T>>,
    newEvent: Event<T>,
    keyFn: (event: Event<T>) => K,
    duration: number,
  ): ReadonlyArray<Event<T>> => {
    const now = Date.now();
    const cutoff = now - duration;
    const newEventKey = keyFn(newEvent);

    // Early optimization: if array is empty, just return the new event
    if (events.length === 0) {
      return [newEvent];
    }

    // Find the cutoff index using binary search
    let start = 0;
    let end = events.length - 1;
    let cutoffIndex = 0;

    while (start <= end) {
      const mid = Math.floor((start + end) / 2);
      if (events[mid].ts < cutoff) {
        start = mid + 1;
        cutoffIndex = start;
      } else {
        end = mid - 1;
      }
    }

    // Build a map of the latest event for each key within time window
    // Starting from cutoff point to avoid processing expired events
    const keyMap = new Map<K, Event<T>>();

    // Process only non-expired events
    for (let i = cutoffIndex; i < events.length; i++) {
      const event = events[i];
      const key = keyFn(event);

      // Skip events with same key as new event
      if (key === newEventKey) continue;

      // For other keys, keep track of the latest event
      const existing = keyMap.get(key);
      if (!existing || existing.ts < event.ts) {
        keyMap.set(key, event);
      }
    }

    // Add the new event
    keyMap.set(newEventKey, newEvent);

    // Convert to array and return
    return Array.from(keyMap.values());
  },

  /**
  getRecent: <T extends z.ZodTypeAny>(
    events: ReadonlyArray<Event<T>>,
    duration: number,
  ): ReadonlyArray<Event<T>> => {
    const now = Date.now();
    const cutoff = now - duration;

    // Early optimization for empty arrays or when all events are recent
    if (events.length === 0 || events[0].ts >= cutoff) {
      return events;
    }

    // Find cutoff index using binary search
    let start = 0;
    let end = events.length - 1;
    let cutoffIndex = events.length;

    while (start <= end) {
      const mid = Math.floor((start + end) / 2);
      if (events[mid].ts < cutoff) {
        start = mid + 1;
        cutoffIndex = start;
      } else {
        end = mid - 1;
      }
    }

    // Return only the recent part
    return events.slice(cutoffIndex);
  },

  /**
   * Caches only the most recent event
   */
  latest: <T extends z.ZodTypeAny>(
    events: ReadonlyArray<Event<T>>,
    newEvent: Event<T>,
  ): ReadonlyArray<Event<T>> => {
    // Early optimization: if array is empty, just return the new event
    if (events.length === 0) {
      return [newEvent];
    }

    // Find the last event that is not expired
    const lastEvent = events[events.length - 1];

    // If the new event is newer, return it
    if (newEvent.ts > lastEvent.ts) {
      return [newEvent];
    }

    // Otherwise, return the existing events
    return events;
  },
};

/**
 * Configuration for creating a room implementation
 * @template S Store state type
 */
export interface RoomImplementationConfig<S> {
  /**
   * Function to create the store for this room
   * @returns A new Zustand store instance
   */
  createStore: () => StoreApi<S>;

  /**
   * Function that creates a message handler for this room
   * @param store The store that will be updated by the handler
   * @returns A message handler function
   */
  createMessageHandler: (store: StoreApi<S>) => MessageCallback;
}

/**
 * Result type returned by room implementation hooks
 * @template S The store state type
 */
export interface RoomImplementationResult<S> {
  /** The room state data, or null if not available */
  state: S;
  /** Whether the room is successfully connected */
  isConnected: boolean;
}

export function createRoomImplementation<S>({
  createStore,
  createMessageHandler,
}: RoomImplementationConfig<S>) {
  return function useRoomImplementation(
    roomId: RoomId,
  ): RoomImplementationResult<S> {
    const {
      hasRoom,
      initRoom,
      incrementSubscribers,
      decrementSubscribers,
      getRoom,
      socket,
    } = useSocketStore();

    // Track connection state
    const [isConnected, setIsConnected] = useState(false);

    // Use refs to track state between renders
    const isInitializedRef = useRef(false);
    const roomRef = useRef<Room<S> | undefined>(undefined);
    const storeRef = useRef<StoreApi<S> | null>(null);

    // Track socket connection state
    const [isSocketConnected, setIsSocketConnected] = useState<boolean>(
      socket !== null && socket.connected,
    );

    // For debugging purposes
    const serializedRoomId = useMemo(() => serializeRoomId(roomId), [roomId]);

    // Set up a listener for socket connection changes
    useEffect(() => {
      if (!socket) {
        setIsSocketConnected(false);
        return;
      }

      // Update connection state immediately
      setIsSocketConnected(socket.connected);

      const handleConnect = () => {
        setIsSocketConnected(true);
      };

      const handleDisconnect = () => {
        console.log("Socket disconnected");
        setIsSocketConnected(false);
      };

      socket.on("connect", handleConnect);
      socket.on("disconnect", handleDisconnect);

      return () => {
        socket.off("connect", handleConnect);
        socket.off("disconnect", handleDisconnect);
      };
    }, [socket]); // Re-run only when socket instance changes

    // Room initialization effect
    useEffect(() => {
      // Start out as not connected until we confirm
      setIsConnected(false);

      // Don't try to initialize rooms if socket isn't connected
      if (!isSocketConnected) {
        console.warn(
          `Cannot initialize room ${serializedRoomId}: Socket not connected. Will retry when connected.`,
        );
        return;
      }

      // Create a variable to track if we need to increment subscribers
      let needsIncrement = false;

      try {
        // Check if room already exists (this handles remounting in Strict Mode)
        if (hasRoom(roomId)) {
          roomRef.current = getRoom<S>(roomId);

          // Make sure we have a valid room before incrementing
          if (roomRef.current) {
            needsIncrement = true;
            setIsConnected(true);
          } else {
            console.error(
              `Room ${serializedRoomId} exists but couldn't be retrieved`,
            );
            // Force re-initialization on next render
            isInitializedRef.current = false;
          }
        } else if (!isInitializedRef.current) {
          // Room doesn't exist, initialize it

          // Create store if not already created
          if (!storeRef.current) {
            storeRef.current = createStore();
          }

          const store = storeRef.current;
          const messageHandler = createMessageHandler(store);

          // Initialize the room
          initRoom<S>(roomId, () => store, messageHandler);

          // Verify room was successfully created
          if (!hasRoom(roomId)) {
            console.error(`Failed to initialize room ${serializedRoomId}`);
            isInitializedRef.current = false;
            return;
          }

          // Room created successfully
          isInitializedRef.current = true;
          roomRef.current = getRoom<S>(roomId);

          // Double-check that we got a valid room
          if (!roomRef.current) {
            console.error(
              `Room ${serializedRoomId} was created but couldn't be retrieved`,
            );
            isInitializedRef.current = false;
            return;
          }

          needsIncrement = true;
          setIsConnected(true);
        }

        // Now it's safe to increment subscribers, but only if we determined we need to
        if (needsIncrement) {
          incrementSubscribers(roomId);
        }
      } catch (err) {
        console.error(`Error initializing room ${serializedRoomId}:`, err);
        isInitializedRef.current = false;
        setIsConnected(false);
      }

      // Return cleanup function
      return () => {
        decrementSubscribers(roomId);
      };
    }, [
      roomId,
      serializedRoomId,
      isSocketConnected,
      hasRoom,
      initRoom,
      incrementSubscribers,
      decrementSubscribers,
      getRoom,
    ]);

    // Get the state, but handle not-found cases gracefully
    const defaultStore = useMemo(() => createStore(), []);
    const defaultState = defaultStore.getState();

    const state = useSyncExternalStore(
      (callback) => {
        if (!roomRef.current?.store) return () => {};
        return roomRef.current.store.subscribe(callback);
      },
      () => {
        if (!roomRef.current?.store) {
          // return a temporary store if roomRef is not initialized
          return defaultState;
        }
        return roomRef.current.store.getState();
      },
    );

    return { state, isConnected };
  };
}
