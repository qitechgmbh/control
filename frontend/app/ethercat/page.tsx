"use client";

import { Page } from "@/components/Page";
import { GetEthercat, useClient } from "@/hooks/useClient";
import { useState, useEffect } from "react";

export default function EthercatPage() {
  const client = useClient();
  const [devices, setDevices] = useState<GetEthercat["devices"]>([]);
  useEffect(() => {
    client.getEthercat().then((data) => setDevices(data.devices));
  }, [client]);

  return (
    <Page title="EtherCAT">
      <ul>
        {devices.map((device) => (
          <li key={device.address}>{device.name}</li>
        ))}
      </ul>
    </Page>
  );
}
