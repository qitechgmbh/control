import { DeviceRole } from "@/machines/types";
import React from "react";
import { Value } from "./Value";

type Props = {
  device_role: DeviceRole;
};

export function DeviceRoleComponent({ device_role }: Props) {
  return (
    <>
      <Value value={device_role.role} /> {device_role.role_label}
    </>
  );
}
