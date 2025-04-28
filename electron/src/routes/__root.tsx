import React from "react";
import { Outlet, createRootRoute } from "@tanstack/react-router";

export const RootRoute = createRootRoute({
  component: Root,
});

function Root() {
  return (
    <>
      <main className="h-screen pb-20">
        <Outlet />
      </main>
    </>
  );
}
