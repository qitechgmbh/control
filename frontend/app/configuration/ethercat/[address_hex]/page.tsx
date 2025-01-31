"use client";

import { Page } from "@/components/Page";
import { Button } from "@/components/ui/button";
import { useClient } from "@/hooks/useClient";
import { useParams } from "next/navigation";

export default function DevicePage() {
  const { address_hex } = useParams();
  const client = useClient();

  // address is a hex string
  const address = parseInt(address_hex as string, 16);

  const handleClick = () => {
    client.x({
      address,
    });
  };

  // get path from router
  return (
    <Page>
      <span className="text-xl"> Device {address_hex}</span>
      <Button onClick={handleClick}>X</Button>
    </Page>
  );
}
