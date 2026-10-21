import { create } from "zustand";
import { persist } from "zustand/middleware";

export type UpdateNotificationState = {
  // Release the user chose to skip; never prompted for again
  skippedVersion: string | null;
};

export type UpdateNotificationActions = {
  setSkippedVersion: (version: string | null) => void;
};

export type UpdateNotificationStore = UpdateNotificationState &
  UpdateNotificationActions;

export const useUpdateNotificationStore = create<UpdateNotificationStore>()(
  persist(
    (set) => ({
      skippedVersion: null,

      setSkippedVersion: (version: string | null) =>
        set({ skippedVersion: version }),
    }),
    {
      name: "update-notification-storage",
    },
  ),
);
