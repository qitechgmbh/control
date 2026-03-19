import React from "react";
import { createRoot } from "react-dom/client";
import App from "@ui/App";
import { Toaster } from "@ui/components/ui/sonner";

const root = createRoot(document.getElementById("app")!);
root.render(
  <React.StrictMode>
    <App />
    <Toaster />
  </React.StrictMode>,
);
