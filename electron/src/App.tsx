import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import { syncThemeWithLocal } from "./helpers/theme_helpers";
import { useTranslation } from "react-i18next";
import "./localization/i18n";
import { updateAppLanguage } from "./helpers/language_helpers";
import { router } from "./routes/router";
import { RouterProvider } from "@tanstack/react-router";
import { Toaster } from "./components/ui/sonner";
import { useSocketStore } from "./client/socketioStore";
import { enableMapSet } from "immer";

export default function App() {
  const { i18n } = useTranslation();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

  // connect socketio
  const { connect } = useSocketStore();
  useEffect(() => {
    connect("http://localhost:3001/");
  }, [connect]);

  return <RouterProvider router={router} />;
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
