import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import { syncThemeWithLocal } from "./helpers/theme_helpers";
import { useTranslation } from "react-i18next";
import "./localization/i18n";
import { updateAppLanguage } from "./helpers/language_helpers";
import { router } from "./routes/router";
import { RouterProvider } from "@tanstack/react-router";
import { Toaster } from "./components/ui/sonner";
import { enableMapSet } from "immer";
import { GlobalLaserToastManager } from "./setup/GlobalLaserToastManager";
import { startMemoryMonitor } from "./client/memoryMonitor";
import { installFrontendDiagnostics } from "./client/frontendDiagnostics";

export default function App() {
  const { i18n } = useTranslation();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

  useEffect(() => {
    return startMemoryMonitor();
  }, []);

  useEffect(() => {
    return installFrontendDiagnostics();
  }, []);

  return (
    <>
      <GlobalLaserToastManager />
      <RouterProvider router={router} />
    </>
  );
}

// enable immer MapSet plugin
enableMapSet();

const root = createRoot(document.getElementById("app")!);
root.render(
  <React.StrictMode>
    <App />
    <Toaster />
  </React.StrictMode>,
);
