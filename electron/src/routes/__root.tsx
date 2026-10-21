import React from "react";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import { UpdateAvailableDialog } from "@/setup/UpdateAvailableDialog";

export const RootRoute = createRootRoute({
  component: Root,
});

function Root() {
  return (
    <>
      <main className="h-screen pb-20">
        <Outlet />
      </main>
      <UpdateAvailableDialog />
    </>
  );
}
