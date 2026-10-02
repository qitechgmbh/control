import React, { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import { useEffectAsync } from "@/lib/useEffectAsync";
import { useGithubSourceStore } from "@/stores/githubSourceStore";
import { useUpdateStore } from "@/stores/updateStore";
import { useUpdateNotificationStore } from "@/stores/updateNotificationStore";

// Check only once per app start, so "Not now" lasts until the next launch
let checkedThisSession = false;

type AvailableUpdate = {
  current: string;
  latest: string;
};

export function UpdateAvailableDialog() {
  const navigate = useNavigate();
  const { githubSource } = useGithubSourceStore();
  const { setSkippedVersion } = useUpdateNotificationStore();
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);

  useEffectAsync(async () => {
    if (checkedThisSession) return;
    checkedThisSession = true;

    const result = await window.update.checkLatestRelease(githubSource);
    if ("error" in result) {
      console.warn(result.error);
      return;
    }
    if (!result.current || !result.latest) return;
    if (result.latest === useUpdateNotificationStore.getState().skippedVersion)
      return;
    if (useUpdateStore.getState().isUpdating) return;

    setUpdate({ current: result.current, latest: result.latest });
  }, []);

  if (!update) return null;

  const close = () => setUpdate(null);

  return (
    <Dialog open onOpenChange={(open) => !open && close()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New version available</DialogTitle>
          <DialogDescription className="text-base">
            Version <span className="font-mono">{update.latest}</span> is
            available. You are currently running{" "}
            <span className="font-mono">{update.current}</span>.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <TouchButton variant="outline" onClick={close}>
            Not now
          </TouchButton>
          <TouchButton
            variant="outline"
            onClick={() => {
              setSkippedVersion(update.latest);
              close();
            }}
          >
            Skip this version
          </TouchButton>
          <TouchButton
            icon="lu:CircleArrowUp"
            onClick={() => {
              close();
              navigate({
                to: "/_sidebar/setup/update/changelog",
                search: {
                  tag: update.latest,
                  ...githubSource,
                },
              });
            }}
          >
            View update
          </TouchButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
