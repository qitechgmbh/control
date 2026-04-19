import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import { syncThemeWithLocal } from "./helpers/theme_helpers";
import { useTranslation } from "react-i18next";
import "./localization/i18n";
import { updateAppLanguage } from "./helpers/language_helpers";
import { router } from "./routes/router";
import { RouterProvider, useNavigate } from "@tanstack/react-router";
import { Toaster } from "./components/ui/sonner";
import { enableMapSet } from "immer";
import { GlobalLaserToastManager } from "./setup/GlobalLaserToastManager";
import { useUpdate } from "./lib/update/useUpdate";

export default function App() {
  const { i18n } = useTranslation();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

  const { isUpdating, currentUpdateInfo } = useUpdate();
  const navigate = useNavigate();

  if (isUpdating) {
    navigate({
      to: "/_sidebar/setup/update/execute",
      search: currentUpdateInfo!,
    });
  }

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
