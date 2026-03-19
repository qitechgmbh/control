import { useEffect } from "react";
import { syncThemeWithLocal } from "@ui/helpers/theme_helpers";
import { useTranslation } from "react-i18next";
import "@ui/localization/i18n";
import { updateAppLanguage } from "@ui/helpers/language_helpers";
import { router } from "@ui/routes/router";
import { RouterProvider } from "@tanstack/react-router";
import { enableMapSet } from "immer";
import { GlobalLaserToastManager } from "@ui/setup/GlobalLaserToastManager";

// enable immer MapSet plugin
enableMapSet();

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
