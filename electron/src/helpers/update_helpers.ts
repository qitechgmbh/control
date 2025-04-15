export async function updatExecute(
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
  return new Promise(async (resolve) => {
    window.update.onLog(onLog);
    window.update.execute(source);
    window.update.onEnd((params) => {
      window.update.onLog(() => {});
      window.update.onEnd(() => {});
      resolve(params);
    });
  });
}
