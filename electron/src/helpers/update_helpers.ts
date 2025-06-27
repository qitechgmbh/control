let updateCancelled = false;

export function cancelCurrentUpdate() {
  updateCancelled = true;
  // Also signal the backend to cancel
  window.update.cancel();
}

export function resetUpdateCancellation() {
  updateCancelled = false;
}

export async function updateExecute(
  source: {
    githubRepoOwner: string;
    githubRepoName: string;
    githubToken?: string;
    tag?: string;
    branch?: string;
    commit?: string;
  },
  onLog: (log: string) => void,
  shouldCancel?: () => boolean,
): Promise<{ success: boolean; error?: string }> {
  // Reset cancellation state when starting a new update
  updateCancelled = false;

  return new Promise((resolve) => {
    const originalOnLog = onLog;
    const wrappedOnLog = (log: string) => {
      // Only check cancellation if we're not already cancelled and shouldCancel is provided
      if (updateCancelled) {
        resolve({ success: false, error: "Update was cancelled by user" });
        return;
      }

      // Check shouldCancel only if it's provided and we're not already resolved
      if (shouldCancel && shouldCancel()) {
        updateCancelled = true;
        resolve({ success: false, error: "Update was cancelled by user" });
        return;
      }

      originalOnLog(log);
    };

    window.update.onLog(wrappedOnLog);
    window.update.execute(source);
    window.update.onEnd((params) => {
      window.update.onLog(() => {});
      window.update.onEnd(() => {});

      if (updateCancelled || (shouldCancel && shouldCancel())) {
        resolve({ success: false, error: "Update was cancelled by user" });
      } else {
        resolve(params);
      }
    });
  });
}
