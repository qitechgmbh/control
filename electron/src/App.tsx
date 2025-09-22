import React, { useEffect,Profiler } from "react";
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

/*
function onRender(id, phase, actualDuration, baseDuration, startTime, commitTime) {
  console.log(id,phase,actualDuration,baseDuration,startTime,commitTime)
}*/

function onRender(id, phase, actualDuration) {
  if (actualDuration > 10) { // threshold in ms
    console.warn(`${id} ${phase} took ${actualDuration.toFixed(1)}ms`);
  }
}


export default function App() {
  const { i18n } = useTranslation();

  // Start global log streaming
  useGlobalLogStreaming();

  useEffect(() => {
    syncThemeWithLocal();
    updateAppLanguage(i18n);
  }, [i18n]);

  return <RouterProvider router={router} />;
}

// enable immer MapSet plugin
enableMapSet();

const root = createRoot(document.getElementById("app")!);

root.render(
  <React.StrictMode>
<Profiler id="App" onRender={onRender}>
    <App />
</Profiler>
    <Toaster />
  </React.StrictMode>,

);
