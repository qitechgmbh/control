import { create } from "zustand";
import { persist } from "zustand/middleware";
import { GithubSource, defaultGithubSource } from "@/setup/GithubSourceDialog";

export type GithubSourceState = {
  githubSource: GithubSource;
};

export type GithubSourceActions = {
  setGithubSource: (source: GithubSource) => void;
  resetGithubSource: () => void;
};

export type GithubSourceStore = GithubSourceState & GithubSourceActions;

export const useGithubSourceStore = create<GithubSourceStore>()(
  persist(
    (set) => ({
      githubSource: defaultGithubSource,

      setGithubSource: (source: GithubSource) => set({ githubSource: source }),

      resetGithubSource: () => set({ githubSource: defaultGithubSource }),
    }),
    {
      name: "github-source-storage",
    },
  ),
);
