"use strict";

import {
  MachineDeviceIdentification,
  MachineIdentificationUnique,
  Option,
} from "@/lib/types";
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
  warning: string | undefined;
  status: "no_data" | "error" | "warning" | "success";
  ts: number;
};

export function useSockerioEvent<T>(
  room: string,
  event: string,
): MessageResponse<T> {
  const socket = useSocketioRoom(room);
  const [res, setRes] = useState<MessageResponse<T>>({
    event,
    data: undefined,
    error: undefined,
    warning: undefined,
    status: "no_data",
    ts: Date.now(),
  });

  useEffect(() => {
    if (socket) {
      socket.on(event, (res) => {
        console.log("event", event, res);
        setRes(res);
      });
    }
  }, [socket, event]);

  return res;
}

export type EthercatSetupEvent = {
  devices: {
    configured_address: number;
    name: string;
    vendor_id: number;
    product_id: number;
    revision: number;
    machine_device_identification: Option<MachineDeviceIdentification>;
    subdevice_index: number;
  }[];
  machines: {
    machine_identification_unique: MachineIdentificationUnique;
    error: Option<string>;
  }[];
};

export type EthercatSetupEventDevice = EthercatSetupEvent["devices"][0];
export type EthercatSetupEventMachine = EthercatSetupEvent["machines"][0];

export function useSocketioEthercatSetupEvent(): MessageResponse<EthercatSetupEvent> {
  return useSockerioEvent<EthercatSetupEvent>("main", "EthercatSetupEvent");
}
