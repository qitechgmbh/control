import { create } from "zustand";

export type KioskState = {
  // Blocks all touch and keyboard input except the hold-to-unlock on the logo
  locked: boolean;
};

export type KioskActions = {
  setLocked: (locked: boolean) => void;
};

export type KioskStore = KioskState & KioskActions;

// Not persisted: a restart always comes up unlocked
export const useKioskStore = create<KioskStore>()((set) => ({
  locked: false,

  setLocked: (locked: boolean) => {
    set({ locked });
    window.kiosk.setLocked(locked);
  },
}));
