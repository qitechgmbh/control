import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import { syncThemeWithLocal } from "@ui/helpers/theme_helpers";
import { useTranslation } from "react-i18next";
import "@ui/localization/i18n";
import { updateAppLanguage } from "@ui/helpers/language_helpers";
import { router } from "@ui/routes/router";
import { RouterProvider } from "@tanstack/react-router";
import { Toaster } from "@ui/components/ui/sonner";
import { enableMapSet } from "immer";
import { GlobalLaserToastManager } from "@ui/setup/GlobalLaserToastManager";

export default function App() {
  const { i18n } = useTranslation();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

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
