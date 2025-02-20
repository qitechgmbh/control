import React from "react";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import DragWindowRegion from "@/components/DragWindowRegion";

export const RootRoute = createRootRoute({
  component: Root,
});

function Root() {
  return (
    <>
      <DragWindowRegion />
      <main className="h-screen pb-20">
        <Outlet />
      </main>
    </>
  );
}
