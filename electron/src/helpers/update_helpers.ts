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
): Promise<{ success: boolean; error?: string }> {
  return new Promise((resolve) => {
    // Clean up any existing listeners first
    window.update.onLog(() => {});
    window.update.onEnd(() => {});

    // Set up new listeners
    window.update.onLog(onLog);
    window.update.onEnd((params) => {
      // Clean up listeners when done
      window.update.onLog(() => {});
      window.update.onEnd(() => {});
      resolve(params);
    });

    // Start the update
    window.update.execute(source);
  });
}
