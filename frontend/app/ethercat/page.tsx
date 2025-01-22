"use client";

import { Page } from "@/components/Page";
import {
  useSocketioEthercatDevicesEvent,
  useSocketioRoom,
} from "@/hooks/useSocketio";

export default function EthercatPage() {
  const deviceMessage = useSocketioEthercatDevicesEvent();
  // const socket = useSocketioRoom("main");
  // socket?.on("EthercatDevicesEvent", (res) => {
  //   console.log(res);
  // });

  return (
    <Page title="EtherCAT">
      {deviceMessage.data?.devices.map((device) => (
        <div key={device.adress}>
          <div>{device.adress}</div>
          <div>{device.name}</div>
        </div>
      ))}
      {deviceMessage.error && <div>{deviceMessage.error}</div>}
    </Page>
  );
}
