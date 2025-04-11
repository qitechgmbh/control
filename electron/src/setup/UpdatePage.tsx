import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { Value } from "@/components/Value";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useMemo } from "react";
import { getVendorPreset, getMachinePreset } from "@/machines/types";
import { IconText } from "@/components/IconText";
import {
  EthercatSetupEventData,
  useMainNamespace,
} from "@/client/mainNamespace";

export function UpdatePage() {
  return (
    <Page>
      <SectionTitle title="Update"></SectionTitle>
    </Page>
  );
}
