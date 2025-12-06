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
import { useGlobalLogStreaming } from "./hooks/useGlobalLogStreaming";
import { useSystemKeyboard } from "./hooks/useSystemKeyboard";
import { VirtualKeyboardProvider } from "./contexts/VirtualKeyboardContext";

function AppContent() {
  const { i18n } = useTranslation();

  // Start global log streaming
  useGlobalLogStreaming();

  // Ensure virtual keyboard appears when inputs are focused
  useSystemKeyboard();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

  return <RouterProvider router={router} />;
}

export default function App() {
  return (
    <VirtualKeyboardProvider>
      <AppContent />
    </VirtualKeyboardProvider>
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
