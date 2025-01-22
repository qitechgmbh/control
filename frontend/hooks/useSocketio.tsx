"use strict";

import { useEffect, useState } from "react";
import { io } from "socket.io-client";

export function useSocketio() {
  const [isConnecting, setIsConnecting] = useState(false);
  const [socket, setSocket] = useState<ReturnType<typeof io> | null>(null);

  useEffect(() => {
    if (!isConnecting && !socket) {
      console.log("connecting");
      setIsConnecting(true);

      const socket = io("http://localhost:3001/");
      socket.on("connect", () => {
        console.log("connected");
        setSocket(socket);
        setIsConnecting(false);
      });
      socket.on("disconnect", () => {
        console.log("disconnected");
        setSocket(null);
        setIsConnecting(false);
      });
    }
  }, [isConnecting, socket]);

  return socket;
}

export function useSocketioRoom(room: string) {
  const socket = useSocketio();
  const [isRoomJoined, setIsRoomJoined] = useState(false);

  useEffect(() => {
    if (socket && !isRoomJoined) {
      socket.emit("join", {
        room,
      });
      setIsRoomJoined(true);
    }
    // return () => {
    //   if (socket) {
    //     socket.emit("leave", {
    //       room,
    //     });
    //     setIsRoomJoined(false);
    //   }
    // };
  }, [socket, room, isRoomJoined]);

  return socket;
}

export type MessageResponse<T> = {
  event: string;
  data: T | undefined;
  error: string | undefined;
  ts: number;
};

export function useSockerioEvent<T>(
  room: string,
  event: string
): MessageResponse<T> {
  const socket = useSocketioRoom(room);
  const [res, setRes] = useState<MessageResponse<T>>({
    event,
    data: undefined,
    error: "No Data",
    ts: Date.now(),
  });

  useEffect(() => {
    if (socket) {
      socket.on(event, (res) => {
        setRes(res);
      });
    }
  }, [socket, event]);

  return res;
}

export type EthercatDevicesEvent = {
  devices: {
    adress: number;
    name: string;
  }[];
};

export function useSocketioEthercatDevicesEvent(): MessageResponse<EthercatDevicesEvent> {
  return useSockerioEvent<EthercatDevicesEvent>("main", "EthercatDevicesEvent");
}
